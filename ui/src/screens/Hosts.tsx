import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useHosts, type SavedHost } from "@/state/hosts";
import { Button } from "@/components/Button";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { FAB } from "@/components/FAB";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function Hosts() {
  const { hosts, remove } = useHosts();
  const nav = useNavigate();
  const [pending, setPending] = useState<SavedHost | null>(null);

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
                      setPending(h);
                    }}
                    aria-label="Remove host"
                    className="text-xs text-[var(--text-tertiary)] hover:text-[var(--error)] px-3 py-2 active:opacity-70 shrink-0"
                  >
                    remove
                  </button>
                }
              />
            </div>
          ))}
        </div>
      )}
      <ConfirmDialog
        open={pending !== null}
        onOpenChange={(o) => { if (!o) setPending(null); }}
        title={pending ? `Remove ${pending.label}?` : ""}
        description="The host is removed from this device only — the nub it points at keeps running."
        confirmLabel="Remove"
        destructive
        onConfirm={() => { if (pending) remove(pending.hid); setPending(null); }}
      />
    </Page>
  );
}
