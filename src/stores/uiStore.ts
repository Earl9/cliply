import { useCallback, useState } from "react";

export type UiState = {
  detailsFocused: boolean;
  monitoringPaused: boolean;
  windowPinned: boolean;
};

export const defaultUiState: UiState = {
  detailsFocused: false,
  monitoringPaused: false,
  windowPinned: false,
};

export function useUiStore() {
  const [state, setState] = useState(defaultUiState);

  const setWindowPinned = useCallback((windowPinned: boolean) => {
    setState((current) => ({ ...current, windowPinned }));
  }, []);

  const setMonitoringPaused = useCallback((monitoringPaused: boolean) => {
    setState((current) => ({ ...current, monitoringPaused }));
  }, []);

  return {
    ...state,
    setWindowPinned,
    setMonitoringPaused,
  };
}
