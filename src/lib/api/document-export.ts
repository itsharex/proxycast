import { safeInvoke } from "@/lib/dev-bridge/safeInvoke";

export async function saveExportedDocument(
  filePath: string,
  content: string,
): Promise<void> {
  await safeInvoke("save_exported_document", {
    filePath,
    content,
  });
}
