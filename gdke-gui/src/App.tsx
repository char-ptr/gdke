import { useEffect, useState } from "react";
import "./App.css";
import { open } from "@tauri-apps/plugin-dialog";
import { event } from "@tauri-apps/api";
import { invoke } from "@tauri-apps/api/core";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
  DialogHeader,
} from "./components/ui/dialog";
import { Label } from "./components/ui/label";
import { Input } from "./components/ui/input";
import { Button } from "./components/ui/button";

function App() {
  const [drag_over, set_drag_over] = useState(false);
  const [show_pre_run, set_show_pre_run] = useState(false);
  const [signature, set_signature] = useState("");
  const [program, set_program] = useState("");
  const [loading, set_loading] = useState(false);
  const [outcome, set_outcome] = useState<null | [true, string] | [false]>(
    null,
  );
  async function get_file() {
    const outcome = await open({
      title: "File picker",
      filters: [{ name: "Applications", extensions: ["exe"] }],
      directory: false,
    });
    if (!outcome) return;
    ask_for_sig(outcome.path);
  }
  const run_program = () => {
    set_loading(true);
    invoke("get_secret", { sig: signature, program }).then(
      (out) => {
        set_outcome([true, out as string]);
        set_loading(false);
      },
      (e) => {
        set_loading(false);
        set_outcome([false]);
      },
    );
  };
  const ask_for_sig = async (path: string) => {
    set_outcome(null);
    set_signature("");
    set_show_pre_run(true);
    set_program(path);
  };
  useEffect(() => {
    const listeners: Promise<event.UnlistenFn>[] = [];
    listeners.push(
      event.listen<{ paths: string[] }>(event.TauriEvent.DRAG_DROP, (evt) => {
        set_drag_over(false);
        console.log(evt);
        ask_for_sig(evt.payload.paths[0]);
      }),
    );
    listeners.push(
      event.listen(event.TauriEvent.DRAG_LEAVE, (evt) => {
        set_drag_over(false);
        console.log(evt);
      }),
    );
    listeners.push(
      event.listen(event.TauriEvent.DRAG_ENTER, (evt) => {
        set_drag_over(true);
        console.log(evt);
      }),
    );
    return () => {
      (async () => {
        for await (const off_fn of listeners) {
          off_fn();
        }
      })();
    };
  }, []);

  //captybara testing is real
  return (
    <div
      className={`h-full py-5 container dark:bg-black dark:text-white ${drag_over ? "border-dotted min-h-screen rounded-lg border-2 border-neutral-300 dark:border-neutral-700" : ""}`}
    >
      <Dialog open={show_pre_run} onOpenChange={(e) => set_show_pre_run(e)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Extract Secret Key</DialogTitle>
            {outcome === null && !loading && (
              <DialogDescription>
                After entering a sig we will try our hardest to search for the
                key with that signature.
              </DialogDescription>
            )}
          </DialogHeader>
          <div className="flex gap-3 flex-col">
            {outcome === null || loading ? (
              <div>
                <Label>Signature</Label>
                <Input
                  onChange={(e) => set_signature(e.currentTarget.value)}
                  placeholder="59 48 29 ?? 92 47 ?? .."
                />
              </div>
            ) : loading ? (
              <div>Loading just give us a sec</div>
            ) : outcome?.[0] === true ? (
              <>
                <p className="text-neutral-300 tracking-wide">
                  Successfully got Key!
                </p>{" "}
                <Label>Secret Key</Label>{" "}
                <Input readOnly={true} value={outcome[1]} />
              </>
            ) : (
              <div>failure</div>
            )}
            <div className="ml-auto flex flex-row gap-3">
              {outcome?.[0] && (
                <Button
                  onClick={() => navigator.clipboard.writeText(outcome[1])}
                  variant={"ghost"}
                  disabled={signature.length === 0}
                >
                  Copy Key
                </Button>
              )}
              <Button
                onClick={
                  outcome === null ? run_program : () => set_show_pre_run(false)
                }
                disabled={signature.length === 0}
              >
                {outcome === null ? "Ready" : "Close"}
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
      <h1 className="text-2xl tracking-tight">👋 Welcome to Gdke</h1>
      <p className="text-neutral-400 text-lg tracking-wide">
        To get started drop your program below:
      </p>

      <div className="mt-6 items-center justify-center w-full">
        <label
          htmlFor="dropzone-file"
          className="flex flex-col items-center justify-center w-full h-64 border-2 border-gray-300 border-dashed rounded-lg cursor-pointer bg-neutral-50-50 dark:hover:bg-neutral-800 dark:bg-neutral-900 hover:bg-neutral-100 dark:border-neutral-600 dark:hover:border-neutral-500"
        >
          <div className="flex flex-col items-center justify-center pt-5 pb-6">
            <svg
              className="w-8 h-8 mb-4 text-gray-500 dark:text-gray-400"
              aria-hidden="true"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 20 16"
            >
              <path
                stroke="currentColor"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 13h3a3 3 0 0 0 0-6h-.025A5.56 5.56 0 0 0 16 6.5 5.5 5.5 0 0 0 5.207 5.021C5.137 5.017 5.071 5 5 5a4 4 0 0 0 0 8h2.167M10 15V6m0 0L8 8m2-2 2 2"
              />
            </svg>
            <p className="mb-2 text-sm text-gray-500 dark:text-gray-400">
              <span className="font-semibold">Click to upload</span> or drag and
              drop
            </p>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              EXE, Application files
            </p>
          </div>
          <button
            onClick={() => get_file()}
            id="dropzone-file"
            type="button"
            className="hidden"
          />
        </label>
      </div>
    </div>
  );
}

export default App;
