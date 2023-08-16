import React, { useEffect } from "react";
import {
  FluentProvider,
  teamsLightTheme,
  makeStyles,
} from "@fluentui/react-components";
import { BrowserRouter, Route, Routes } from "react-router-dom";

import "./App.css";
import Home from "./pages/Home";
import { useGlobalState } from "./store/state";
import { getConfig } from "./api/api";

const useStyles = makeStyles({
  root: {
    display: "flex",
    flexDirection: "column",
    flexWrap: "nowrap",
    width: "100%",
    backgroundColor: "whitesmoke",
  },
});

function App() {
  const classes = useStyles();

  const setConfig = useGlobalState((state) => state.setConfig);

  useEffect(() => {
    getConfig()
      .then((config) => setConfig(config))
      .catch(console.error);
  }, [setConfig]);

  return (
    <>
      <React.StrictMode>
        <div className={classes.root}>
          <div
            style={{
              display: "flex",
              flexDirection: "row",
              justifyContent: "center",
              alignItems: "flex-start",
              height: "100%",
              width: "100%",
            }}
          >
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                justifyContent: "center",
                alignItems: "center",
                height: "100%",
                width: "80%",
                maxWidth: "40rem",
              }}
            >
              <FluentProvider className={classes.root} theme={teamsLightTheme}>
                <BrowserRouter>
                  <Routes>
                    <Route path="/" Component={Home} index />
                  </Routes>
                </BrowserRouter>
              </FluentProvider>
            </div>
          </div>
        </div>
      </React.StrictMode>
    </>
  );
}

export default App;
