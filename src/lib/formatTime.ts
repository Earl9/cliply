const shortFormatter = new Intl.DateTimeFormat("en-US", {
  hour: "2-digit",
  minute: "2-digit",
});

const fullFormatter = new Intl.DateTimeFormat("en-US", {
  month: "short",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
});

export function formatCopiedTime(value: string) {
  return shortFormatter.format(new Date(value));
}

export function formatFullCopiedTime(value: string) {
  return fullFormatter.format(new Date(value));
}

export function formatRelativeTime(value: string, now = new Date("2026-05-04T10:43:00+08:00")) {
  const copiedAt = new Date(value);
  const diffMs = now.getTime() - copiedAt.getTime();
  const minutes = Math.max(0, Math.round(diffMs / 60_000));

  if (minutes < 1) {
    return "Just now";
  }

  if (minutes < 60) {
    return `${minutes} min ago`;
  }

  const hours = Math.round(minutes / 60);
  if (hours < 24) {
    return `${hours} hr ago`;
  }

  return fullFormatter.format(copiedAt);
}

export function formatBytes(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }

  const kb = bytes / 1024;
  if (kb < 1024) {
    return `${kb.toFixed(kb >= 100 ? 0 : 1)} KB`;
  }

  const mb = kb / 1024;
  return `${mb.toFixed(mb >= 100 ? 0 : 1)} MB`;
}
