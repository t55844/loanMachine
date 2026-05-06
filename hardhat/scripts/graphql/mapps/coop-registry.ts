import { CoopRegistered } from "../generated/CoopRegistry/CoopRegistry"
import { LoanMachine } from "../generated/templates"
import { Cooperative, CoopRegisteredEvent } from "../generated/schema"
import { ethereum, BigInt } from "@graphprotocol/graph-ts"

function makeId(event: ethereum.Event): string {
  return event.transaction.hash.toHex() + "-" + event.logIndex.toString()
}

function formatTimestamp(ts: BigInt): string {
  let ms = ts.times(BigInt.fromI32(1000)).toI64()
  let dt = new Date(ms)
  let d  = dt.getUTCDate().toString();        if (d.length  == 1) d  = "0" + d
  let m  = (dt.getUTCMonth() + 1).toString(); if (m.length  == 1) m  = "0" + m
  let h  = dt.getUTCHours().toString();       if (h.length  == 1) h  = "0" + h
  let mi = dt.getUTCMinutes().toString();     if (mi.length == 1) mi = "0" + mi
  let s  = dt.getUTCSeconds().toString();     if (s.length  == 1) s  = "0" + s
  return d + "/" + m + "/" + dt.getUTCFullYear().toString() + " " + h + ":" + mi + ":" + s
}

export function handleCoopRegistered(event: CoopRegistered): void {
  // Cooperative.id == lowercased LoanMachine address
  let coopEntityId = event.params.loanMachine.toHexString()

  let coop = new Cooperative(coopEntityId)
  coop.coopId       = event.params.coopId
  coop.name         = event.params.name
  coop.loanMachine  = event.params.loanMachine
  coop.registeredAt = event.block.timestamp
  coop.active       = true
  coop.save()

  let log = new CoopRegisteredEvent(makeId(event))
  log.coopId          = event.params.coopId
  log.loanMachine     = event.params.loanMachine
  log.name            = event.params.name
  log.blockTimestamp  = formatTimestamp(event.block.timestamp)
  log.transactionHash = event.transaction.hash
  log.save()

  // Spawn the dynamic LoanMachine indexer for this coop
  LoanMachine.create(event.params.loanMachine)
}