import React, { useEffect, useState } from "react";

import {
  Option,
  Combobox,
  Field,
  Input,
  makeStyles,
  Button,
  tokens,
  Switch,
  Dialog,
  DialogBody,
  DialogTitle,
  DialogActions,
  DialogSurface,
  DialogContent,
  Spinner,
} from "@fluentui/react-components";
import { EyeRegular, EyeOffRegular } from "@fluentui/react-icons";

import { dialog } from "@tauri-apps/api";

import { useGlobalState } from "../store/state";
import {
  DavServerStatus,
  checkDavServer,
  startDavServer,
  stopDavServer,
  updateConfig,
} from "../api/api";

const useStyles = makeStyles({
  col: {
    display: "flex",
    flexBasis: "12",
    flexDirection: "column",
    justifyContent: "flex-start",
    alignContent: "center",
    alignItems: "center",
    flexWrap: "wrap",
    rowGap: tokens.spacingVerticalM,
  },
  row: {
    display: "flex",
    flexBasis: "12",
    flexDirection: "row",
    justifyContent: "flex-start",
    alignContent: "center",
    alignItems: "center",
    flexWrap: "wrap",
    columnGap: tokens.spacingHorizontalM,
  },
  item: {
    // display: "flex",
  },
  noEyeSee: {
    "::-ms-reveal": {
      display: "none",
    },
  },
});

export default function Home() {
  const classes = useStyles();

  // const state = useGlobalState();
  const config = useGlobalState((state) => state.config);
  const setConfig = useGlobalState((state) => state.setConfig);

  // auth states
  const [userLocal, setUserLocal] = useState("");
  const [passwordLocal, setPasswordLocal] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [enableAuth, setEnableAuth] = useState(false);
  useEffect(() => {
    if (enableAuth) {
      setConfig((config) => ({ ...config, auth: [userLocal, passwordLocal] }));
    } else {
      setConfig((config) => ({
        ...config,
        auth: undefined,
      }));
    }
  }, [enableAuth, userLocal, passwordLocal, setConfig]);

  // server states
  const [running, setRunning] = useState(false);
  const [processing, setProcessing] = useState(false);

  useEffect(() => {
    if (!processing) {
      const timer = setInterval(() => {
        checkDavServer().then((res) => {
          if (res == DavServerStatus.Running) {
            setRunning(true);
          } else {
            setRunning(false);
          }
        });
      }, 2000);
      return () => clearInterval(timer);
    }
  }, [processing]);

  // control dialog
  const [dialogDom, setDialogDom] = useState<React.JSX.Element | null>(null);

  const setIp = (ip: string) => setConfig({ ...config, ip });
  const setPort = (port: number) => setConfig({ ...config, port });
  const setRoot = (root: string) => setConfig({ ...config, root });

  const enableAuthSwitchCb = () => {
    if (enableAuth) {
      setEnableAuth(!enableAuth);
    } else {
      if (userLocal && passwordLocal) {
        setEnableAuth(!enableAuth);
      } else {
        console.log("user and password must set to enable login");

        setDialogDom(
          <Dialog modalType="alert" open>
            <DialogSurface>
              <DialogBody>
                <DialogTitle>Warning</DialogTitle>
                <DialogContent>
                  User and Password must set to enable login.
                </DialogContent>
                <DialogActions>
                  <Button
                    appearance="primary"
                    onClick={() => setDialogDom(null)}
                  >
                    OK
                  </Button>
                </DialogActions>
              </DialogBody>
            </DialogSurface>
          </Dialog>
        );
      }
    }
  };

  return (
    <>
      {dialogDom}

      <div className={classes.row}>
        <div className={classes.item} style={{ flex: 6 }}>
          <Field label="IP">
            <Combobox
              value={config?.ip ?? ""}
              onChange={(e) => setIp(e.target.value)}
              onOptionSelect={(_, v) => setIp(v.optionText ?? "")}
              placeholder="*"
              appearance="underline"
              freeform
            >
              {["127.0.0.1", "[::]", "0.0.0.0"].map((v) => (
                <Option key={v} text={v}>
                  {v}
                </Option>
              ))}
            </Combobox>
          </Field>
        </div>
        <div
          className={classes.row}
          style={{
            flex: 2,
            alignSelf: "flex-end",
            justifyContent: "center",
          }}
        >
          <span
            className={classes.item}
            style={{
              // padding: "0.5rem 0.5rem",
              lineHeight: "2.3rem",
              fontWeight: "bold",
              fontSize: "1.2rem",
            }}
          >
            :
          </span>
        </div>
        <div className={classes.item} style={{ flex: 4 }}>
          <Field label="Port">
            <Input
              value={config?.port?.toString() ?? ""}
              onChange={(e) => setPort(Number(e.target.value))}
              placeholder={config?.enableTls ? "443" : "80"}
              appearance="underline"
            />
          </Field>
        </div>
      </div>

      <div
        className={classes.row}
        style={{
          justifyContent: "space-between",
          alignContent: "stretch",
          alignItems: "flex-end",
        }}
      >
        <div
          className={classes.item}
          style={{ flex: 8, maxWidth: "75%", flexShrink: 1 }}
        >
          <Field label="Select directory">
            <Input appearance="underline" value={config?.root ?? ""} />
          </Field>
        </div>

        <div
          className={classes.item}
          style={{ flex: 1, maxWidth: "20%", flexShrink: 0.5 }}
        >
          <Button
            appearance="primary"
            onClick={() => {
              dialog
                .open({
                  title: "Select Dav directory",
                  directory: true,
                })
                .then((path) => {
                  if (path) {
                    setRoot(path as string);
                  }
                })
                .catch(console.error);
            }}
          >
            Browse
          </Button>
        </div>
      </div>

      <div
        className={classes.row}
        style={{
          justifyContent: "space-between",
          alignContent: "stretch",
          alignItems: "flex-end",
        }}
      >
        <div
          className={classes.item}
          style={{
            flex: 4,
            maxWidth: "40%",
            flexShrink: 1,
          }}
        >
          <Field label="user">
            <Input
              appearance="underline"
              onChange={(e) => setUserLocal(e.target.value)}
              value={userLocal}
            />
          </Field>
        </div>

        <div
          className={classes.item}
          style={{
            flex: 4,
            maxWidth: "40%",
            flexShrink: 1,
          }}
        >
          <Field label="password">
            <Input
              input={{ className: classes.noEyeSee }}
              className={showPassword ? "" : classes.noEyeSee}
              appearance="underline"
              onChange={(e) => setPasswordLocal(e.target.value)}
              value={passwordLocal}
              contentAfter={
                <Button
                  icon={showPassword ? <EyeRegular /> : <EyeOffRegular />}
                  appearance="transparent"
                  onClick={() => setShowPassword(!showPassword)}
                />
              }
              type={showPassword ? "text" : "password"}
            />
          </Field>
        </div>

        <div
          className={classes.item}
          style={{
            flex: 4,
            maxWidth: "10%",
            flexShrink: 0,
          }}
        >
          <Field label="login">
            <Switch checked={enableAuth} onChange={enableAuthSwitchCb} />
          </Field>
        </div>
      </div>

      <div
        className={classes.row}
        style={{
          justifyContent: "flex-end",
        }}
      >
        <div className={classes.item}>
          {
            <Button
              appearance="primary"
              icon={
                processing ? (
                  <Spinner labelPosition="before" size="tiny" />
                ) : null
              }
              onClick={
                running
                  ? // stop server
                    () => {
                      if (processing) return;
                      setProcessing(true);
                      stopDavServer()
                        .then(() => setRunning(false))
                        .catch((err) => {
                          console.log(err);
                          setDialogDom(
                            <Dialog modalType="alert" open>
                              <DialogSurface>
                                <DialogBody>
                                  <DialogTitle>Error</DialogTitle>
                                  <DialogContent>{err}</DialogContent>
                                  <DialogActions>
                                    <Button
                                      appearance="primary"
                                      onClick={() => setDialogDom(null)}
                                    >
                                      OK
                                    </Button>
                                  </DialogActions>
                                </DialogBody>
                              </DialogSurface>
                            </Dialog>
                          );
                        })
                        .finally(() => setProcessing(false));
                    }
                  : // start server
                    () => {
                      if (processing) return;
                      console.log("start server", config);
                      setProcessing(true);
                      updateConfig(config!)
                        .then(() => startDavServer())
                        .then(() => console.log("server started"))
                        .then(() => setRunning(true))
                        .catch((err) => {
                          console.log(err);
                          setDialogDom(
                            <Dialog modalType="alert" open>
                              <DialogSurface>
                                <DialogBody>
                                  <DialogTitle>Error</DialogTitle>
                                  <DialogContent>{err}</DialogContent>
                                  <DialogActions>
                                    <Button
                                      appearance="primary"
                                      onClick={() => setDialogDom(null)}
                                    >
                                      OK
                                    </Button>
                                  </DialogActions>
                                </DialogBody>
                              </DialogSurface>
                            </Dialog>
                          );
                        })
                        .finally(() => setProcessing(false));
                    }
              }
            >
              {running ? "Stop" : "Start"}
            </Button>
          }
        </div>
      </div>
    </>
  );
}
