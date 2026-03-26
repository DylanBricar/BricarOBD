import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { nav } from "./nav";
import { connection } from "./connection";
import { dashboard } from "./dashboard";
import { liveData } from "./liveData";
import { dtc } from "./dtc";
import { mode06 } from "./mode06";
import { freezeFrame } from "./freezeFrame";
import { ecu } from "./ecu";
import { monitors } from "./monitors";
import { history } from "./history";
import { advanced } from "./advanced";
import { status } from "./status";
import { common } from "./common";
import { devConsole } from "./devConsole";
import { demo } from "./demo";

const resources = {
  fr: {
    translation: {
      ...nav.fr,
      ...connection.fr,
      ...dashboard.fr,
      ...liveData.fr,
      ...dtc.fr,
      ...mode06.fr,
      ...freezeFrame.fr,
      ...ecu.fr,
      ...monitors.fr,
      ...history.fr,
      ...advanced.fr,
      ...status.fr,
      ...common.fr,
      ...devConsole.fr,
      ...demo.fr,
    },
  },
  en: {
    translation: {
      ...nav.en,
      ...connection.en,
      ...dashboard.en,
      ...liveData.en,
      ...dtc.en,
      ...mode06.en,
      ...freezeFrame.en,
      ...ecu.en,
      ...monitors.en,
      ...history.en,
      ...advanced.en,
      ...status.en,
      ...common.en,
      ...devConsole.en,
      ...demo.en,
    },
  },
};

i18n.use(initReactI18next).init({
  resources,
  lng: "fr",
  fallbackLng: "en",
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
