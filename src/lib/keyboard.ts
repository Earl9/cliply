export function isEditableElement(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  const tagName = target.tagName.toLowerCase();
  return tagName === "input" || tagName === "textarea" || target.isContentEditable;
}

export function clampIndex(index: number, length: number) {
  if (length <= 0) {
    return -1;
  }

  return Math.min(Math.max(index, 0), length - 1);
}
