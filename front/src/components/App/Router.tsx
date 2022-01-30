import * as React from "react"

import type { Route, RouteId } from "~/misc/constants"

import { ChainProvider } from "./ChainProvider"
import { Top } from "../pages/Top"
import { Mtc } from "../pages/Mtc"
import { EmoBases } from "../pages/EmoBases"
import { MtcTrial } from "../pages/MtcTrial"
import { Dev } from "../pages/Dev"
import { MtcDebug } from "../pages/MtcDebug"
import { EmoAbilityBuilder } from "../pages/EmoAbilityBuilder"
import { Style } from "../pages/Style"

export function Router(props: { route: Route }) {
  const [kind, e] = getElement(props.route.id)

  switch (kind) {
    case "chain":
      return <ChainProvider>{e}</ChainProvider>
    case "contract":
      return e
    case "none":
      return e
  }
}

const getElement = (routeId: RouteId): ["chain" | "contract" | "none", JSX.Element] => {
  switch (routeId) {
    case "/":
      return ["chain", <Top />]
    case "/match":
      return ["chain", <Mtc />]
    case "/emo_bases":
      return ["chain", <EmoBases />]
    case "/match_trial":
      return ["chain", <MtcTrial />]
    case "/dev":
      return ["chain", <Dev />]
    case "/match_debug":
      return ["chain", <MtcDebug />]
    case "/emo_ability_builder":
      return ["chain", <EmoAbilityBuilder />]
    case "/style":
      return ["none", <Style />]
    case "/match_contract":
      return ["contract", <Style />] // TODO: change
    case "/not_found":
    default:
      return ["none", <h1>page not found</h1>]
  }
}
