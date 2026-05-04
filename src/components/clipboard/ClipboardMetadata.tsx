import { Badge } from "@/components/common/Badge";

const metadata = [
  ["Source app", "Visual Studio Code"],
  ["Copied", "Today, 10:42:18"],
  ["Type", "Code"],
  ["Size", "148 bytes"],
  ["Pinned", "Yes"],
];

export function ClipboardMetadata() {
  return (
    <div className="mt-4 rounded-lg border border-[color:var(--cliply-border)] bg-white/64 p-4">
      <h3 className="mb-3 text-sm font-semibold text-[color:var(--cliply-text)]">Metadata</h3>
      <dl className="grid grid-cols-[120px_minmax(0,1fr)] gap-x-4 gap-y-2 text-sm">
        {metadata.map(([label, value]) => (
          <div key={label} className="contents">
            <dt className="text-[color:var(--cliply-muted)]">{label}</dt>
            <dd className="min-w-0 text-[color:var(--cliply-text)]">
              {label === "Type" ? <Badge tone="accent">{value}</Badge> : value}
            </dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
