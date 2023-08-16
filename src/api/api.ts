import { invoke } from "@tauri-apps/api";

export type Config = {
  ip?: string;
  port?: number;
  root?: string;

  auth?: [user: string, password: string];
  enableTls?: boolean;
};

export async function getConfig(): Promise<Config> {
  return await invoke("get_config");
}

export async function updateConfig(config: Config): Promise<void> {
  return await invoke("update_config", { config });
}

export type CertConfig = {
  certPath?: string;
  keyPath?: string;
}

export async function updateCertConfig(config: CertConfig): Promise<void> {
  if (config.certPath === undefined && config.keyPath === undefined) {
    throw new Error("Either certPath or keyPath must set");
  }
  return await invoke("import_tls_or_cert_from_path", config);
}

export async function startDavServer(): Promise<void> {
  return await invoke("start_dav_server");
}

export async function stopDavServer(): Promise<void> {
  return await invoke("stop_dav_server");
}

export enum DavServerStatus {
  Stopped = "stopped",
  Running = "running",
}

export async function checkDavServer(): Promise<DavServerStatus> {
  return await invoke("check_dav_server");
}
