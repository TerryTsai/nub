import { Link } from "react-router-dom";
import { useHosts } from "@/state/hosts";
import { Button } from "@/components/Button";
import { Card } from "@/components/Card";

export function Hosts() {
  const { hosts, remove } = useHosts();

  return (
    <Page title="nub">
      {hosts.length === 0 ? (
        <Card>
          <div className="py-2 flex flex-col gap-3 items-start">
            <p className="text-[var(--text-secondary)]">
              No hosts yet. Connect to your first nub to get started.
            </p>
            <Link to="/add"><Button>Add host</Button></Link>
          </div>
        </Card>
      ) : (
        <>
          {hosts.map((h) => (
            <Card key={h.hid}>
              <div className="flex justify-between items-start gap-3">
                <Link to={`/h/${h.hid}`} className="flex-1 min-w-0 -m-1 p-1">
                  <div className="text-base font-semibold truncate">{h.label}</div>
                  <div className="text-xs text-[var(--text-tertiary)] mono truncate">{h.url}</div>
                </Link>
                <Button
                  variant="ghost"
                  onClick={() => {
                    if (confirm(`Remove ${h.label}?`)) remove(h.hid);
                  }}
                  aria-label="Remove host"
                  className="text-sm"
                >
                  Remove
                </Button>
              </div>
            </Card>
          ))}
          <Link to="/add" className="self-start mt-1">
            <Button variant="ghost">+ Add another</Button>
          </Link>
        </>
      )}
    </Page>
  );
}

export function Page({ title, children, right }: { title: string; children: React.ReactNode; right?: React.ReactNode }) {
  return (
    <div className="min-h-full p-5 max-w-md mx-auto flex flex-col gap-3">
      <div className="flex items-baseline justify-between px-1 mb-1">
        <h1 className="text-3xl font-semibold tracking-tight">{title}</h1>
        {right}
      </div>
      {children}
    </div>
  );
}
