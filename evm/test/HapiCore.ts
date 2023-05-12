import { loadFixture } from "@nomicfoundation/hardhat-network-helpers";
import { expect } from "chai";
import { ethers } from "hardhat";
import { v4 as uuid } from "uuid";

import { setupContract } from "./setup";
import { ReporterRole, ReporterStatus, randomId } from "./util";

describe("HapiCore", function () {
  async function basicFixture() {
    let setup = await setupContract();

    const [owner, authority, nobody] = await ethers.getSigners();

    return { ...setup, owner, authority, nobody };
  }

  describe("Deployment", function () {
    it("Should set the right owner and authority", async function () {
      const { hapiCore, owner } = await loadFixture(basicFixture);

      expect(await hapiCore.owner()).to.equal(owner.address);

      expect(await hapiCore.authority()).to.equal(owner.address);
    });

    it("Should correctly set authority from owner", async function () {
      const { hapiCore, authority } = await loadFixture(basicFixture);

      await expect(await hapiCore.setAuthority(authority.address))
        .to.emit(hapiCore, "AuthorityChanged")
        .withArgs(authority.address);

      expect(await hapiCore.authority()).to.equal(authority.address);
    });

    it("Should correctly set authority from previous authority", async function () {
      const { hapiCore, authority, nobody } = await loadFixture(basicFixture);

      await expect(await hapiCore.setAuthority(authority.address))
        .to.emit(hapiCore, "AuthorityChanged")
        .withArgs(authority.address);

      expect(await hapiCore.authority()).to.equal(authority.address);

      await expect(
        await hapiCore.connect(authority).setAuthority(nobody.address)
      )
        .to.emit(hapiCore, "AuthorityChanged")
        .withArgs(nobody.address);

      expect(await hapiCore.authority()).to.equal(nobody.address);
    });

    it("Should not allow setting authority from non-owner/non-authority", async function () {
      const { hapiCore, authority, nobody } = await loadFixture(basicFixture);

      await expect(
        hapiCore.connect(nobody).setAuthority(authority.address)
      ).to.be.revertedWith("Caller is not the owner or authority");
    });
  });

  describe("Configuration", function () {
    it("Should update stake configuration", async function () {
      const { hapiCore } = await loadFixture(basicFixture);

      expect(await hapiCore.stakeConfiguration()).to.deep.equal([
        ethers.constants.AddressZero,
        0,
        0,
        0,
        0,
        0,
      ]);

      const stakeTokenAddress = "0xdEADBEeF00000000000000000000000000000000";

      await expect(
        await hapiCore.updateStakeConfiguration(
          stakeTokenAddress,
          3600,
          101,
          102,
          103,
          104
        )
      )
        .to.emit(hapiCore, "StakeConfigurationChanged")
        .withArgs(stakeTokenAddress, 3600, 101, 102, 103, 104);

      expect(await hapiCore.stakeConfiguration()).to.deep.equal([
        stakeTokenAddress,
        3600,
        101,
        102,
        103,
        104,
      ]);
    });

    it("Should update reward configuration", async function () {
      const { hapiCore } = await loadFixture(basicFixture);

      expect(await hapiCore.rewardConfiguration()).to.deep.equal([
        ethers.constants.AddressZero,
        0,
        0,
      ]);

      const rewardTokenAddress = "0xdEADBEeF00000000000000000000000000000000";

      await expect(
        await hapiCore.updateRewardConfiguration(rewardTokenAddress, 101, 102)
      )
        .to.emit(hapiCore, "RewardConfigurationChanged")
        .withArgs(rewardTokenAddress, 101, 102);

      expect(await hapiCore.rewardConfiguration()).to.deep.equal([
        rewardTokenAddress,
        101,
        102,
      ]);
    });
  });

  describe("Reporter management", function () {
    it("Should create a reporter", async function () {
      const { hapiCore } = await loadFixture(basicFixture);

      const reporter = {
        account: "0xdEADBEeF00000000000000000000000000000000",
        id: randomId(),
        role: ReporterRole.Publisher,
        name: "publisher",
        url: "https://publisher.blockchain",
      };

      await expect(
        await hapiCore.createReporter(
          reporter.id,
          reporter.account,
          reporter.role,
          reporter.name,
          reporter.url
        )
      )
        .to.emit(hapiCore, "ReporterCreated")
        .withArgs(reporter.id, reporter.account, reporter.role);

      expect(await hapiCore.getReporter(reporter.id)).to.deep.equal([
        reporter.id,
        reporter.account,
        reporter.name,
        reporter.url,
        reporter.role,
        ReporterStatus.Inactive,
        0,
        0,
      ]);
    });

    it("Should not create a reporter if not authority", async function () {
      const { hapiCore, nobody } = await loadFixture(basicFixture);

      const reporter = {
        account: "0xdEADBEeF00000000000000000000000000000000",
        id: randomId(),
        role: ReporterRole.Publisher,
        name: "publisher",
        url: "https://publisher.blockchain",
      };

      await expect(
        hapiCore
          .connect(nobody)
          .createReporter(
            reporter.id,
            reporter.account,
            reporter.role,
            reporter.name,
            reporter.url
          )
      ).to.be.revertedWith("Caller is not the authority");
    });

    it("Should update a reporter", async function () {
      const { hapiCore } = await loadFixture(basicFixture);

      const reporterOld = {
        account: "0xdEADBEeF00000000000000000000000000000000",
        id: randomId(),
        role: ReporterRole.Publisher,
        name: "publisher",
        url: "https://publisher.blockchain",
      };

      const reporterNew = {
        id: reporterOld.id,
        account: "0xb04b26349DE3f1B4Dc2e54ecCb54458c343C2909",
        role: ReporterRole.Authority,
        name: "authority",
        url: "https://authority.blockchain",
      };

      await hapiCore.createReporter(
        reporterOld.id,
        reporterOld.account,
        reporterOld.role,
        reporterOld.name,
        reporterOld.url
      );

      await expect(
        await hapiCore.updateReporter(
          reporterOld.id,
          reporterNew.account,
          reporterNew.role,
          reporterNew.name,
          reporterNew.url
        )
      )
        .to.emit(hapiCore, "ReporterUpdated")
        .withArgs(reporterOld.id, reporterNew.account, reporterNew.role);

      expect(await hapiCore.getReporter(reporterOld.id)).to.deep.equal([
        reporterOld.id,
        reporterNew.account,
        reporterNew.name,
        reporterNew.url,
        reporterNew.role,
        ReporterStatus.Inactive,
        0,
        0,
      ]);
    });

    it("Should not update a reporter if not authority", async function () {
      const { hapiCore, nobody } = await loadFixture(basicFixture);

      const reporter = {
        account: "0xdEADBEeF00000000000000000000000000000000",
        id: randomId(),
        role: ReporterRole.Publisher,
        name: "publisher",
        url: "https://publisher.blockchain",
      };

      await hapiCore.createReporter(
        reporter.id,
        reporter.account,
        reporter.role,
        reporter.name,
        reporter.url
      );

      await expect(
        hapiCore
          .connect(nobody)
          .updateReporter(
            reporter.id,
            reporter.account,
            reporter.role,
            reporter.name,
            reporter.url
          )
      ).to.be.revertedWith("Caller is not the authority");
    });
  });
});
