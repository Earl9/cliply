import { Code2 } from "lucide-react";

const code = `const user = await getProfile(session.userId);

if (!user?.enabled) {
  return createEmptySession();
}`;

export function ClipboardPreview() {
  return (
    <div className="rounded-lg border border-[color:var(--cliply-border)] bg-white/78 p-4 shadow-sm">
      <div className="mb-3 flex items-center gap-2 text-sm font-semibold text-[color:var(--cliply-text)]">
        <Code2 className="size-4 text-[color:var(--cliply-accent)]" />
        TypeScript snippet
      </div>
      <pre className="overflow-auto rounded-md bg-[#172033] p-4 text-sm leading-6 text-slate-100">
        <code>
          {code.split("\n").map((line, index) => (
            <span key={`${line}-${index}`} className="block">
              <span className="mr-4 inline-block w-5 select-none text-right text-slate-500">
                {index + 1}
              </span>
              {line || " "}
            </span>
          ))}
        </code>
      </pre>
    </div>
  );
}
