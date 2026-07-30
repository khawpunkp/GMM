import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import type { ArchiveAnalysis, ImportArchiveRequest } from "../types";

function fallbackNameFromPath(path: string): string {
  const filename = path.split(/[\\/]/).pop() ?? "New Mod";
  return filename.replace(/\.(zip|7z|rar)$/i, "");
}

/**
 * One-click import for pages that already have a fixed destination (an agent or category page) —
 * skips ImportModal's full form: pick an archive, analyze it, confirm just the name via a plain
 * prompt(), then import straight into `target`. Returns whether a mod was actually imported.
 */
export async function quickImportMod(target: { agentId?: number; categoryId?: number }): Promise<boolean> {
  const path = await open({
    multiple: false,
    filters: [{ name: "Mod archive", extensions: ["zip", "7z", "rar"] }],
  });
  if (typeof path !== "string") return false;

  let deducedName = fallbackNameFromPath(path);
  let deducedAuthor: string | null = null;
  let selectedRoot: string | null = null;

  try {
    const analysis = await invoke<ArchiveAnalysis>("analyze_archive", { archivePath: path });
    deducedName = analysis.deducedName ?? deducedName;
    deducedAuthor = analysis.deducedAuthor ?? null;
    selectedRoot = analysis.entries.find((e) => e.isLikelyModRoot)?.path ?? null;
  } catch (e) {
    alert(String(e));
    return false;
  }

  const name = prompt("Name this mod:", deducedName);
  if (!name || !name.trim()) return false;

  try {
    const request: ImportArchiveRequest = {
      archivePath: path,
      agentId: target.agentId ?? null,
      categoryId: target.categoryId ?? null,
      categoryItemId: null,
      selectedInternalRoot: selectedRoot,
      modName: name.trim(),
      description: null,
      author: deducedAuthor,
    };
    await invoke("import_archive", { request });
    return true;
  } catch (e) {
    alert(String(e));
    return false;
  }
}
