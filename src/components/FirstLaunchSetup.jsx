// src/components/FirstLaunchSetup.jsx
import React, { useState, useEffect, useCallback } from "react";
import { useSettings } from "../contexts/SettingsContext";
import { invoke } from "@tauri-apps/api/tauri";
import { toast } from "react-toastify";
import GameSwitcher from "./GameSwitcher"; // Import our new component
import {
  FileIcon,
  FloppyDiskIcon,
  FolderOpenIcon,
} from "@phosphor-icons/react";

function FirstLaunchSetup() {
  const {
    modsFolder: initialModsFolder,
    quickLaunchPath: initialQuickLaunch,
    updateSetting,
    fetchSettings,
    SETTINGS_KEY_MODS_FOLDER,
    SETTINGS_KEY_QUICK_LAUNCH,
  } = useSettings();

  // Local state for the setup screen
  const [selectedModsFolder, setSelectedModsFolder] = useState(
    initialModsFolder || ""
  );
  const [selectedQuickLaunch, setSelectedQuickLaunch] = useState(
    initialQuickLaunch || ""
  );
  const [currentGameForSetup, setCurrentGameForSetup] = useState(""); // The game currently active
  const [isSwitchingGame, setIsSwitchingGame] = useState(false);
  const [gameLoadError, setGameLoadError] = useState("");
  const [gameSwitchError, setGameSwitchError] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState("");

  // Fetch available games and the *actual* current game on mount
  useEffect(() => {
    let isMounted = true;
    setGameLoadError("");
    setCurrentGameForSetup("zzz");
    return () => {
      isMounted = false;
    };
  }, []);

  // Update local path state if context values change after initial load
  useEffect(() => {
    setSelectedModsFolder(initialModsFolder || "");
    setSelectedQuickLaunch(initialQuickLaunch || "");
  }, [initialModsFolder, initialQuickLaunch]);

  // Handle game switch
  const handleGameSwitch = async (targetGameSlug) => {
    if (targetGameSlug === currentGameForSetup || isSwitchingGame) {
      return;
    }

    setIsSwitchingGame(true);
    setGameSwitchError("");
    toast.info(
      `Switching to ${targetGameSlug.toUpperCase()} and restarting setup...`
    );

    try {
      // This command triggers the restart
      await invoke("switch_game", { targetGameSlug });
      // If successful, the app restarts, and this component re-mounts
      // with the new 'currentGameForSetup' fetched from the backend.
    } catch (err) {
      const errorString =
        typeof err === "string" ? err : err?.message || "Unknown switch error";
      console.error(
        "Failed to initiate game switch during setup:",
        errorString
      );
      setGameSwitchError(`Failed to switch: ${errorString}`);
      toast.error(`Failed to switch game: ${errorString}`);
      setIsSwitchingGame(false); // Re-enable interaction
    }
    // No 'finally' needed here as success = restart
  };

  const handleSelectModsFolder = async () => {
    setSaveError("");
    try {
      const result = await invoke("select_directory");
      if (result) {
        // Check if user selected something (didn't cancel)
        setSelectedModsFolder(result);
      }
    } catch (err) {
      console.error("Error selecting directory:", err);
      setSaveError(`Failed to select folder: ${err}`);
    }
  };

  const handleSelectQuickLaunch = async () => {
    setSaveError("");
    try {
      const result = await invoke("select_file");
      if (result) {
        // Check if user selected something (didn't cancel)
        setSelectedQuickLaunch(result);
      }
    } catch (err) {
      console.error("Error selecting file:", err);
      setSaveError(`Failed to select file: ${err}`);
    }
  };

  const handleSave = async () => {
    if (!selectedModsFolder) {
      setSaveError("The Mods Folder path must be selected before continuing.");
      return;
    }
    setIsSaving(true);
    setSaveError("");

    try {
      const saveMods = await updateSetting(
        SETTINGS_KEY_MODS_FOLDER,
        selectedModsFolder
      );
      // Allow saving even if quick launch is empty
      const saveLaunch = await updateSetting(
        SETTINGS_KEY_QUICK_LAUNCH,
        selectedQuickLaunch || ""
      );

      if (saveMods && saveLaunch !== false) {
        // Check explicitly for false failure
        await fetchSettings(); // Reload settings in context
        // App.jsx will handle showing the main UI now
      } else {
        throw new Error("One or more settings failed to save.");
      }
    } catch (err) {
      console.error("Save error:", err);
      setSaveError(`Failed to save settings: ${err.message || err}`);
    } finally {
      setIsSaving(false);
    }
  };

  const isActionDisabled = isSaving || isSwitchingGame;
  const canSave = selectedModsFolder && !isActionDisabled;
  const logoSrc = `/images/logos/${currentGameForSetup || "default"}.png`;
  const handleLogoError = (e) => {
    e.target.src = "/images/logos/default.png";
  };

  return (
    <div className="flex justify-center items-center h-dvh w-dvw bg-main-dark">
      <div className="bg-main-gray p-10 rounded-xl max-w-180 w-full text-center flex-col flex gap-4">
        {/* --- Game Logo and Title --- */}
        <div className="flex flex-col gap-5 items-center">
          <img
            src={logoSrc}
            alt={`ZZZ Logo`}
            className="size-25 object-contain rounded-lg"
            onError={handleLogoError}
          />
          <h1 className="font-medium text-white text-3xl">
            Initial Setup for{" "}
            {currentGameForSetup ? currentGameForSetup.toUpperCase() : "..."}
          </h1>
        </div>
        {/* --- End Game Logo --- */}

        <p className="text-sm text-white font-medium">
          Please select the main <b className="text-main-yellow">Mods</b>{" "}
          folder. <span className="text-error">(This is required.)</span>
          <br />
          Optionally, select the launcher executable for Quick Launch.
        </p>

        {/* Mods Folder Selection */}
        <div className="flex items-center gap-4 text-left p-4 rounded-lg bg-main-dark/20">
          <label className="font-medium w-35 shrink-0 flex text-white">
            Mods Folder:
          </label>
          <div
            className="px-4 py-2 grow bg-main-dark/30 line-clamp-1 overflow-hidden text-ellipsis whitespace-nowrap rounded-lg text-white"
            title={selectedModsFolder}
          >
            {selectedModsFolder || "Not Selected"}
          </div>
          <button
            onClick={handleSelectModsFolder}
            disabled={isActionDisabled}
            className="btn btn-outline"
          >
            <FolderOpenIcon size={24} weight="fill" /> Select Folder
          </button>
        </div>

        {/* Quick Launch Selection */}
        <div className="flex items-center gap-4 text-left p-4 rounded-lg bg-main-dark/20">
          <label className="font-medium w-35 shrink-0 flex text-white">
            Quick Launch:
          </label>
          <div
            className="px-4 py-2 grow bg-main-dark/30 line-clamp-1 overflow-hidden text-ellipsis whitespace-nowrap rounded-lg text-white"
            title={selectedQuickLaunch}
          >
            {selectedQuickLaunch || "Not Selected"}
          </div>
          <button
            onClick={handleSelectQuickLaunch}
            disabled={isActionDisabled}
            className="btn btn-outline"
          >
            <FileIcon size={24} weight="fill" /> Select File
          </button>
        </div>

        {/* Error Display */}
        {saveError && (
          <p className="text-error font-medium text-base">{saveError}</p>
        )}

        {/* Save Button */}
        <button
          onClick={handleSave}
          disabled={!canSave}
          className="btn btn-primary mt-6"
        >
          {isSaving ? (
            <>
              <div className="loader size-6" /> Saving...
            </>
          ) : (
            <>
              <FloppyDiskIcon size={24} weight="fill" /> Save & Continue
            </>
          )}
        </button>
      </div>
    </div>
  );
}

export default FirstLaunchSetup;
