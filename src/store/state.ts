import { create } from "zustand";

import { Config } from "../api/api";

export interface GlobalState {
  config?: Config;
  certPath?: {
    certPath?: string;
    keyPath?: string;
  };

  setConfig: (config: Config | ((config?: Config) => Config)) => void;
  setCertPath: (certPath: { certPath?: string; keyPath?: string }) => void;
}

export const useGlobalState = create<GlobalState>()((set, _get) => ({
  setConfig: (config) => {
    set(({ config: old }) => ({
      config: typeof config === "function" ? config(old) : config,
    }));
  },
  setCertPath: (certPath: { certPath?: string; keyPath?: string }) =>
    set(({ certPath: c }) => ({
      certPath: {
        certPath: certPath.certPath ?? c?.certPath,
        keyPath: certPath.keyPath ?? c?.keyPath,
      },
    })),
}));
