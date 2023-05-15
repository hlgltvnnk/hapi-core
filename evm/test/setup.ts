import { ethers, upgrades } from "hardhat";

import { HapiCore } from "../typechain-types";
import { IERC20 } from "../typechain-types";
import { ReporterRole, randomId } from "./util";

export async function setupContract(): Promise<{ hapiCore: HapiCore }> {
  const HapiCore = await ethers.getContractFactory("HapiCore");

  const contract = await upgrades.deployProxy(HapiCore, [], {
    initializer: "initialize",
  });

  await contract.deployed();

  return { hapiCore: contract as HapiCore };
}

export async function basicFixture() {
  let setup = await setupContract();

  const [owner, authority, nobody] = await ethers.getSigners();

  return { ...setup, owner, authority, nobody };
}

export async function fixtureWithToken() {
  let setup = await setupContract();

  const [owner, authority, publisher, validator, nobody] =
    await ethers.getSigners();

  const wallets = { owner, authority, publisher, validator, nobody };

  const cfg = {
    UNLOCK_DURATION: 3600,
    VALIDATOR_STAKE: 101,
    TRACER_STAKE: 102,
    PUBLISHER_STAKE: 103,
    AUTHORITY_STAKE: 104,
  };

  const token = (await ethers.deployContract("Token")) as IERC20;
  await Promise.all([
    token.transfer(authority.address, cfg.AUTHORITY_STAKE * 2),
    token.transfer(publisher.address, cfg.PUBLISHER_STAKE * 2),
    token.transfer(validator.address, cfg.VALIDATOR_STAKE * 2),
    token.transfer(nobody.address, 10000),
    setup.hapiCore.updateStakeConfiguration(
      token.address,
      cfg.UNLOCK_DURATION,
      cfg.VALIDATOR_STAKE,
      cfg.TRACER_STAKE,
      cfg.PUBLISHER_STAKE,
      cfg.AUTHORITY_STAKE
    ),
  ]);

  return {
    ...setup,
    token,
    wallets,
    cfg,
  };
}

export async function fixtureWithReporters() {
  let setup = await fixtureWithToken();

  let { wallets, hapiCore, token, cfg } = setup;

  const reporters = {
    authority: {
      account: wallets.authority.address,
      id: randomId(),
      role: ReporterRole.Authority,
      name: "authority",
      url: "https://authority.blockchain",
    },
    publisher: {
      account: wallets.publisher.address,
      id: randomId(),
      role: ReporterRole.Publisher,
      name: "publisher",
      url: "https://publisher.blockchain",
    },
    validator: {
      account: wallets.validator.address,
      id: randomId(),
      role: ReporterRole.Validator,
      name: "validator",
      url: "https://validator.blockchain",
    },
  };

  await Promise.all([
    hapiCore.createReporter(
      reporters.authority.id,
      reporters.authority.account,
      reporters.authority.role,
      reporters.authority.name,
      reporters.authority.url
    ),
    hapiCore.createReporter(
      reporters.publisher.id,
      reporters.publisher.account,
      reporters.publisher.role,
      reporters.publisher.name,
      reporters.publisher.url
    ),
    hapiCore.createReporter(
      reporters.validator.id,
      reporters.validator.account,
      reporters.validator.role,
      reporters.validator.name,
      reporters.validator.url
    ),
    token
      .connect(wallets.authority)
      .approve(hapiCore.address, cfg.AUTHORITY_STAKE),
    token
      .connect(wallets.publisher)
      .approve(hapiCore.address, cfg.PUBLISHER_STAKE),
    token
      .connect(wallets.validator)
      .approve(hapiCore.address, cfg.VALIDATOR_STAKE),
  ]);

  await Promise.all([
    setup.hapiCore
      .connect(wallets.publisher)
      .activateReporter(reporters.publisher.id),
    setup.hapiCore
      .connect(wallets.validator)
      .activateReporter(reporters.validator.id),
  ]);

  return { ...setup, reporters };
}
