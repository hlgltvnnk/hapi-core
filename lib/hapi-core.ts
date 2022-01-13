import {
  Program,
  web3,
  BN,
  Provider,
  Coder,
  utils,
} from "@project-serum/anchor";
import { NetworkSchemaKeys, pubkeyFromBase58 } from ".";
import { encode as eip55encode } from "eip55";

import { IDL } from "../target/types/hapi_core";
import { bufferFromString, addrToSeeds } from "./buffer";

export function encodeAddress(
  address: string,
  schema: NetworkSchemaKeys
): Buffer {
  let buffer: Buffer = Buffer.from(address);

  switch (schema) {
    case "Ethereum": {
      if (address.match(/^0x/)) {
        address = address.substring(2);
      }
      buffer = Buffer.from(address);
      break;
    }
    case "Solana": {
      buffer = pubkeyFromBase58(address).toBuffer();
      break;
    }
  }

  return buffer;
}

export function decodeAddress(
  address: Buffer,
  schema: NetworkSchemaKeys
): string {
  switch (schema) {
    case "Ethereum": {
      return eip55encode(utils.bytes.hex.encode(address));
    }
    case "Solana": {
      return new web3.PublicKey(address).toBase58();
    }
    default: {
      return address.toString();
    }
  }
}

export function initHapiCore(
  hapiCoreProgramId: string | web3.PublicKey,
  provider?: Provider
) {
  const programId =
    typeof hapiCoreProgramId === "string"
      ? new web3.PublicKey(hapiCoreProgramId)
      : hapiCoreProgramId;

  const program = new Program(IDL, programId, provider);

  const coder = new Coder(IDL);

  async function findCommunityTokenSignerAddress(community: web3.PublicKey) {
    return web3.PublicKey.findProgramAddress(
      [bufferFromString("community_stash"), community.toBytes()],
      programId
    );
  }

  async function findNetworkAddress(community: web3.PublicKey, name: string) {
    return web3.PublicKey.findProgramAddress(
      [
        bufferFromString("network"),
        community.toBytes(),
        bufferFromString(name, 32),
      ],
      programId
    );
  }

  async function findNetworkRewardSignerAddress(network: web3.PublicKey) {
    return web3.PublicKey.findProgramAddress(
      [bufferFromString("network_reward"), network.toBytes()],
      programId
    );
  }

  async function findReporterAddress(
    community: web3.PublicKey,
    pubkey: web3.PublicKey
  ) {
    return web3.PublicKey.findProgramAddress(
      [bufferFromString("reporter"), community.toBytes(), pubkey.toBytes()],
      programId
    );
  }

  async function findReporterRewardAddress(
    network: web3.PublicKey,
    reporter: web3.PublicKey
  ) {
    return web3.PublicKey.findProgramAddress(
      [
        bufferFromString("reporter_reward"),
        network.toBytes(),
        reporter.toBytes(),
      ],
      programId
    );
  }

  async function findCaseAddress(community: web3.PublicKey, caseId: BN) {
    return web3.PublicKey.findProgramAddress(
      [
        bufferFromString("case"),
        community.toBytes(),
        new Uint8Array(caseId.toArray("le", 8)),
      ],
      programId
    );
  }

  async function findAddressAddress(network: web3.PublicKey, address: Buffer) {
    return web3.PublicKey.findProgramAddress(
      [bufferFromString("address"), network.toBytes(), ...addrToSeeds(address)],
      programId
    );
  }

  async function findAssetAddress(
    network: web3.PublicKey,
    mint: Buffer,
    assetId: Buffer | Uint8Array
  ) {
    return web3.PublicKey.findProgramAddress(
      [
        bufferFromString("asset"),
        network.toBytes(),
        ...addrToSeeds(mint),
        assetId,
      ],
      programId
    );
  }

  return {
    ...program,
    programId,
    coder,
    util: {
      encodeAddress,
      decodeAddress,
    },
    pda: {
      findNetworkAddress,
      findNetworkRewardSignerAddress,
      findReporterAddress,
      findReporterRewardAddress,
      findCaseAddress,
      findAddressAddress,
      findAssetAddress,
      findCommunityTokenSignerAddress,
    },
  };
}
