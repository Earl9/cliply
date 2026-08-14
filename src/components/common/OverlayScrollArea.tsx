import {
  useCallback,
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type HTMLAttributes,
  type KeyboardEvent,
  type PointerEvent,
  type Ref,
} from "react";
import { clsx } from "clsx";

const TRACK_INSET = 4;
const MIN_THUMB_HEIGHT = 32;

type ScrollMetrics = {
  clientHeight: number;
  scrollHeight: number;
  scrollTop: number;
  thumbHeight: number;
  thumbTop: number;
  visible: boolean;
};

export type OverlayScrollMetrics = Pick<
  ScrollMetrics,
  "clientHeight" | "scrollHeight" | "scrollTop"
>;

const EMPTY_METRICS: ScrollMetrics = {
  clientHeight: 0,
  scrollHeight: 0,
  scrollTop: 0,
  thumbHeight: 0,
  thumbTop: 0,
  visible: false,
};

type OverlayScrollAreaProps = Omit<HTMLAttributes<HTMLDivElement>, "onScroll"> & {
  scrollbarLabel: string;
  viewportClassName?: string;
  viewportRef?: Ref<HTMLDivElement>;
  onViewportMetricsChange?: (metrics: OverlayScrollMetrics) => void;
};

function assignRef<T>(ref: Ref<T> | undefined, value: T | null) {
  if (typeof ref === "function") {
    ref(value);
  } else if (ref) {
    ref.current = value;
  }
}

export function OverlayScrollArea({
  children,
  className,
  scrollbarLabel,
  viewportClassName,
  viewportRef: forwardedViewportRef,
  onViewportMetricsChange,
  id,
  ...props
}: OverlayScrollAreaProps) {
  const generatedId = useId();
  const viewportId = id ?? `cliply-scroll-${generatedId.replace(/:/g, "")}`;
  const viewportRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const trackRef = useRef<HTMLDivElement>(null);
  const thumbRef = useRef<HTMLDivElement>(null);
  const metricsRef = useRef<ScrollMetrics>(EMPTY_METRICS);
  const metricsFrameRef = useRef<number | null>(null);
  const dragFrameRef = useRef<number | null>(null);
  const pendingDragYRef = useRef<number | null>(null);
  const dragRef = useRef<{
    pointerId: number;
    startY: number;
    startScrollTop: number;
  } | null>(null);
  const metricsCallbackRef = useRef(onViewportMetricsChange);
  const [layoutMetrics, setLayoutMetrics] = useState(EMPTY_METRICS);
  metricsCallbackRef.current = onViewportMetricsChange;

  const setViewportRef = useCallback(
    (node: HTMLDivElement | null) => {
      viewportRef.current = node;
      assignRef(forwardedViewportRef, node);
    },
    [forwardedViewportRef],
  );

  const updateMetricsNow = useCallback(() => {
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }

    // Read all layout values before mutating the scrollbar.
    const { clientHeight, scrollHeight, scrollTop } = viewport;
    const trackHeight = Math.max(0, clientHeight - TRACK_INSET * 2);
    const visible = scrollHeight > clientHeight + 1 && trackHeight > 0;
    const thumbHeight = visible
      ? Math.min(trackHeight, Math.max(MIN_THUMB_HEIGHT, trackHeight * (clientHeight / scrollHeight)))
      : 0;
    const scrollRange = Math.max(1, scrollHeight - clientHeight);
    const thumbTravel = Math.max(0, trackHeight - thumbHeight);
    const thumbTop = visible ? (scrollTop / scrollRange) * thumbTravel : 0;
    const nextMetrics = {
      clientHeight,
      scrollHeight,
      scrollTop,
      thumbHeight,
      thumbTop,
      visible,
    };

    const previousMetrics = metricsRef.current;
    const layoutChanged =
      previousMetrics.clientHeight !== clientHeight ||
      previousMetrics.scrollHeight !== scrollHeight ||
      previousMetrics.thumbHeight !== thumbHeight ||
      previousMetrics.visible !== visible;
    metricsRef.current = nextMetrics;

    const thumb = thumbRef.current;
    const track = trackRef.current;
    if (thumb) {
      thumb.style.transform = `translateY(${thumbTop}px)`;
    }
    if (track) {
      track.setAttribute("aria-valuemax", String(Math.round(Math.max(0, scrollHeight - clientHeight))));
      track.setAttribute("aria-valuenow", String(Math.round(scrollTop)));
    }

    metricsCallbackRef.current?.({ clientHeight, scrollHeight, scrollTop });
    if (layoutChanged) {
      setLayoutMetrics(nextMetrics);
    }
  }, []);

  const scheduleMetricsUpdate = useCallback(() => {
    if (metricsFrameRef.current !== null) {
      return;
    }

    metricsFrameRef.current = window.requestAnimationFrame(() => {
      metricsFrameRef.current = null;
      updateMetricsNow();
    });
  }, [updateMetricsNow]);

  useLayoutEffect(() => {
    const viewport = viewportRef.current;
    const content = contentRef.current;
    if (!viewport || !content) {
      return;
    }

    updateMetricsNow();
    const onScroll = () => scheduleMetricsUpdate();
    viewport.addEventListener("scroll", onScroll, { passive: true });

    const resizeObserver = new ResizeObserver(scheduleMetricsUpdate);
    resizeObserver.observe(viewport);
    resizeObserver.observe(content);

    return () => {
      viewport.removeEventListener("scroll", onScroll);
      resizeObserver.disconnect();
      if (metricsFrameRef.current !== null) {
        window.cancelAnimationFrame(metricsFrameRef.current);
        metricsFrameRef.current = null;
      }
      if (dragFrameRef.current !== null) {
        window.cancelAnimationFrame(dragFrameRef.current);
        dragFrameRef.current = null;
      }
    };
  }, [scheduleMetricsUpdate, updateMetricsNow]);

  const getScrollRange = useCallback(
    () => Math.max(0, metricsRef.current.scrollHeight - metricsRef.current.clientHeight),
    [],
  );
  const getThumbTravel = useCallback(
    () =>
      Math.max(
        1,
        metricsRef.current.clientHeight - TRACK_INSET * 2 - metricsRef.current.thumbHeight,
      ),
    [],
  );

  const applyPendingDrag = useCallback(() => {
    dragFrameRef.current = null;
    const drag = dragRef.current;
    const viewport = viewportRef.current;
    const pointerY = pendingDragYRef.current;
    if (!drag || !viewport || pointerY === null) {
      return;
    }

    const scrollDelta = ((pointerY - drag.startY) / getThumbTravel()) * getScrollRange();
    viewport.scrollTop = drag.startScrollTop + scrollDelta;
  }, [getScrollRange, getThumbTravel]);

  const onThumbPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    pendingDragYRef.current = event.clientY;
    dragRef.current = {
      pointerId: event.pointerId,
      startY: event.clientY,
      startScrollTop: viewport.scrollTop,
    };
  };

  const onThumbPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) {
      return;
    }

    pendingDragYRef.current = event.clientY;
    if (dragFrameRef.current === null) {
      dragFrameRef.current = window.requestAnimationFrame(applyPendingDrag);
    }
  };

  const onThumbPointerUp = (event: PointerEvent<HTMLDivElement>) => {
    if (dragRef.current?.pointerId !== event.pointerId) {
      return;
    }

    if (dragFrameRef.current !== null) {
      window.cancelAnimationFrame(dragFrameRef.current);
      dragFrameRef.current = null;
    }
    pendingDragYRef.current = event.clientY;
    applyPendingDrag();
    dragRef.current = null;
    pendingDragYRef.current = null;
    event.currentTarget.releasePointerCapture(event.pointerId);
  };

  const onTrackPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    const viewport = viewportRef.current;
    if (!viewport || event.target !== event.currentTarget) {
      return;
    }

    event.preventDefault();
    const rect = event.currentTarget.getBoundingClientRect();
    const targetThumbTop = event.clientY - rect.top - metricsRef.current.thumbHeight / 2;
    const ratio = Math.max(0, Math.min(1, targetThumbTop / getThumbTravel()));
    viewport.scrollTop = ratio * getScrollRange();
  };

  const onScrollbarKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }

    const increments: Partial<Record<string, number>> = {
      ArrowUp: -40,
      ArrowDown: 40,
      PageUp: -metricsRef.current.clientHeight * 0.9,
      PageDown: metricsRef.current.clientHeight * 0.9,
    };

    if (event.key === "Home" || event.key === "End") {
      event.preventDefault();
      viewport.scrollTo({ top: event.key === "Home" ? 0 : getScrollRange() });
      return;
    }

    const increment = increments[event.key];
    if (increment !== undefined) {
      event.preventDefault();
      viewport.scrollBy({ top: increment });
    }
  };

  return (
    <div className={clsx("cliply-overlay-scroll-area relative min-h-0 min-w-0 overflow-hidden", className)}>
      <div
        {...props}
        id={viewportId}
        ref={setViewportRef}
        className={clsx("cliply-overlay-scroll-viewport h-full w-full min-h-0 min-w-0 overflow-x-hidden overflow-y-auto", viewportClassName)}
      >
        <div ref={contentRef} className="min-h-full w-full min-w-0 overflow-hidden">
          {children}
        </div>
      </div>
      <div
        ref={trackRef}
        role="scrollbar"
        aria-label={scrollbarLabel}
        aria-controls={viewportId}
        aria-orientation="vertical"
        aria-valuemin={0}
        aria-valuemax={Math.round(Math.max(0, layoutMetrics.scrollHeight - layoutMetrics.clientHeight))}
        aria-valuenow={Math.round(metricsRef.current.scrollTop)}
        tabIndex={layoutMetrics.visible ? 0 : -1}
        hidden={!layoutMetrics.visible}
        className="cliply-overlay-scroll-track"
        onKeyDown={onScrollbarKeyDown}
        onPointerDown={onTrackPointerDown}
      >
        <div
          ref={thumbRef}
          className="cliply-overlay-scroll-thumb"
          style={{ height: layoutMetrics.thumbHeight }}
          onPointerDown={onThumbPointerDown}
          onPointerMove={onThumbPointerMove}
          onPointerUp={onThumbPointerUp}
          onPointerCancel={onThumbPointerUp}
        />
      </div>
    </div>
  );
}
