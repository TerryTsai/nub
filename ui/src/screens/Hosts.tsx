import { Link, useNavigate } from "react-router-dom";
import { useHosts } from "@/state/hosts";
import { Button } from "@/components/Button";
import { Card } from "@/components/Card";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function Hosts() {
  const { hosts, remove } = useHosts();
  const nav = useNavigate();

  return (
    <Page title="Hosts">
      {hosts.length === 0 ? (
        <Card>
          <p className="text-[var(--text-secondary)]">
            No hosts yet. Connect to your first nub to get started.
          </p>
          <Link to="/add" className="self-start">
            <Button>Add host</Button>
          </Link>
        </Card>
      ) : (
        <>
          <div className="flex flex-col">
            {hosts.map((h) => (
              <ListRow
                key={h.hid}
                title={h.label}
                subtitle={h.url}
                onPress={() => nav(`/h/${h.hid}`)}
                right={
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      if (confirm(`Remove ${h.label}?`)) remove(h.hid);
                    }}
                    aria-label="Remove host"
                    className="text-[11px] text-[var(--text-tertiary)] hover:text-[var(--error)] px-1 shrink-0"
                  >
                    remove
                  </button>
                }
              />
            ))}
          </div>
          <Link to="/add" className="self-start mt-1">
            <Button variant="ghost">+ Add another</Button>
          </Link>
        </>
      )}
    </Page>
  );
}
