import { invoke } from "@tauri-apps/api/core";
import { createDesktopCommandClient } from "@prompt-hub/contracts";

export const desktopCommands = createDesktopCommandClient(invoke);
