import { Link, useNavigate } from "react-router-dom";
import { useHosts } from "@/state/hosts";
import { Button } from "@/components/Button";
import { FAB } from "@/components/FAB";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function Hosts() {
  const { hosts, remove } = useHosts();
  const nav = useNavigate();

  const empty = hosts.length === 0;

  return (
    <Page fab={!empty ? <FAB to="/add" label="host" /> : undefined}>
      {empty ? (
        <>
          <p className="text-[var(--text-secondary)] text-sm">
            No hosts yet. Connect to your first nub to get started.
          </p>
          <Link to="/add" className="self-start">
            <Button>Add host</Button>
          </Link>
        </>
      ) : (
        <div className="flex flex-col -mx-1">
          {hosts.map((h) => (
            <div key={h.hid} className="px-1">
              <ListRow
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
            </div>
          ))}
        </div>
      )}
    </Page>
  );
}
