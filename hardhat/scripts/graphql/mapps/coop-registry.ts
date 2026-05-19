import { Bytes, BigInt, ethereum } from "@graphprotocol/graph-ts"

import { CoopRegistered } from "../generated/CoopRegistry/CoopRegistry"
import { LoanMachine as LoanMachineBound } from "../generated/CoopRegistry/LoanMachine"
import { LoanMachine as LoanMachineTemplate } from "../generated/templates"

import {
  Cooperative,
  CoopRegisteredEvent,
  MemberRegisteredEvent
} from "../generated/schema"

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
  return d + "/" + m + "/" + dt.getUTCFullYear().toString()
       + " " + h + ":" + mi + ":" + s
}

function isZeroBytes(b: Bytes): boolean {
  for (let i = 0; i < b.length; i++) {
    if (b[i] != 0) return false
  }
  return true
}

export function handleCoopRegistered(event: CoopRegistered): void {
  // Cooperative.id == lowercased LoanMachine address — other handlers
  // derive coopId from event.address (the LoanMachine), so this MUST
  // match what they'll look up later.
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

  // ── BACKFILL: founder MemberRegisteredEvent ─────────────────
  //
  // initializeMultisig() emits MemberRegistered for the founder BEFORE
  // this CoopRegistered event fires.  The template doesn't exist yet
  // at that point, so loan-machine.ts never sees it.  We reconstruct
  // the missing entity here by reading state directly from the
  // already-initialized contract.
  let lm = LoanMachineBound.bind(event.params.loanMachine)

  let adminsResult = lm.try_getAdmins()
  if (!adminsResult.reverted && adminsResult.value.length > 0) {
    let founder = adminsResult.value[0]

    let memberIdResult = lm.try_getMemberId(founder)
    if (!memberIdResult.reverted && !isZeroBytes(memberIdResult.value)) {
      let entity = new MemberRegisteredEvent(
        event.transaction.hash.toHex() + "-founder-backfill"
      )
      entity.cooperative   = coopEntityId
      entity.wallet        = founder
      entity.memberId      = memberIdResult.value
      entity.isFirstWallet = true
      entity.allWallets    = [founder as Bytes]
      entity.timestamp       = formatTimestamp(event.block.timestamp)
      entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
      entity.transactionHash = event.transaction.hash
      entity.save()
    }
  }

  // Spawn the dynamic indexer.  Anything emitted AFTER this point
  // gets picked up by loan-machine.ts normally.
  LoanMachineTemplate.create(event.params.loanMachine)
}