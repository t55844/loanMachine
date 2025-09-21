const { ethers } = require("hardhat");
const { expect } = require("chai");

describe("LoanMachine", function () {
  let LoanMachine;
  let loanMachine;
  let owner, lender1, lender2, borrower;

  // Convert ether to wei helper
  const toWei = (amount) => ethers.utils.parseEther(amount.toString());
  const fromWei = (amount) => ethers.utils.formatEther(amount);

  beforeEach(async function () {
    [owner, lender1, lender2, borrower] = await ethers.getSigners();
    
    LoanMachine = await ethers.getContractFactory("LoanMachine");
    loanMachine = await LoanMachine.deploy();
    await loanMachine.deployed();
  });

  describe("Donation System", function () {
    it("Should allow users to donate", async function () {
      await expect(loanMachine.connect(lender1).donate({ value: toWei(1) }))
        .to.emit(loanMachine, "Donated")
        .withArgs(lender1.address, toWei(1), toWei(1));
    });

    it("Should update total donations and available balance", async function () {
      await loanMachine.connect(lender1).donate({ value: toWei(1) });
      await loanMachine.connect(lender2).donate({ value: toWei(2) });

      expect(await loanMachine.getTotalDonations()).to.equal(toWei(3));
      expect(await loanMachine.getAvailableBalance()).to.equal(toWei(3));
    });

    it("Should revert when donating 0 ETH", async function () {
      await expect(loanMachine.connect(lender1).donate({ value: 0 }))
        .to.be.revertedWithCustomError(loanMachine, "InvalidAmount");
    });
  });

  describe("Loan Requisition System - Percentage Validation", function () {
    beforeEach(async function () {
      // Setup: lenders donate funds
      await loanMachine.connect(lender1).donate({ value: toWei(10) });
      await loanMachine.connect(lender2).donate({ value: toWei(10) });
    });

    describe("Valid Percentage Range (70-100%)", function () {
      it("Should accept minimum coverage of 70%", async function () {
        await expect(loanMachine.connect(borrower).createLoanRequisition(
          toWei(2), 70, 30
        )).to.emit(loanMachine, "LoanRequisitionCreated");
      });

      it("Should accept minimum coverage of 80%", async function () {
        await expect(loanMachine.connect(borrower).createLoanRequisition(
          toWei(2), 80, 30
        )).to.emit(loanMachine, "LoanRequisitionCreated");
      });

      it("Should accept minimum coverage of 90%", async function () {
        await expect(loanMachine.connect(borrower).createLoanRequisition(
          toWei(2), 90, 30
        )).to.emit(loanMachine, "LoanRequisitionCreated");
      });

      it("Should accept minimum coverage of 100%", async function () {
        await expect(loanMachine.connect(borrower).createLoanRequisition(
          toWei(2), 100, 30
        )).to.emit(loanMachine, "LoanRequisitionCreated");
      });
    });

    describe("Invalid Percentage Range", function () {
      it("Should revert with 0% minimum coverage", async function () {
        await expect(loanMachine.connect(borrower).createLoanRequisition(
          toWei(2), 0, 30
        )).to.be.revertedWithCustomError(loanMachine, "InvalidCoveragePercentage");
      });

      it("Should revert with 50% minimum coverage", async function () {
        await expect(loanMachine.connect(borrower).createLoanRequisition(
          toWei(2), 50, 30
        )).to.be.revertedWithCustomError(loanMachine, "InvalidCoveragePercentage");
      });

      it("Should revert with 69% minimum coverage", async function () {
        await expect(loanMachine.connect(borrower).createLoanRequisition(
          toWei(2), 69, 30
        )).to.be.revertedWithCustomError(loanMachine, "InvalidCoveragePercentage");
      });

      it("Should revert with 101% minimum coverage", async function () {
        await expect(loanMachine.connect(borrower).createLoanRequisition(
          toWei(2), 101, 30
        )).to.be.revertedWithCustomError(loanMachine, "InvalidCoveragePercentage");
      });

      it("Should revert with 150% minimum coverage", async function () {
        await expect(loanMachine.connect(borrower).createLoanRequisition(
          toWei(2), 150, 30
        )).to.be.revertedWithCustomError(loanMachine, "InvalidCoveragePercentage");
      });
    });

    it("Should allow lenders to cover loan requisitions within valid range", async function () {
      // Create requisition with 80% minimum
      const tx = await loanMachine.connect(borrower).createLoanRequisition(
        toWei(2), 80, 30
      );
      const receipt = await tx.wait();
      const requisitionId = receipt.events.find(e => e.event === "LoanRequisitionCreated").args.requisitionId;

      // Lender covers 50% (valid coverage percentage)
      await expect(loanMachine.connect(lender1).coverLoan(requisitionId, 50))
        .to.emit(loanMachine, "LoanCovered")
        .withArgs(requisitionId, lender1.address, toWei(1));

      const requisition = await loanMachine.getRequisitionInfo(requisitionId);
      expect(requisition.currentCoverage).to.equal(50);
      expect(requisition.minimumCoverage).to.equal(80);
    });

    it("Should fund loan when minimum coverage (70%) is reached", async function () {
      const tx = await loanMachine.connect(borrower).createLoanRequisition(
        toWei(2), 70, 30  // 70% minimum
      );
      const receipt = await tx.wait();
      const requisitionId = receipt.events.find(e => e.event === "LoanRequisitionCreated").args.requisitionId;

      // Cover exactly 70% (minimum reached)
      await expect(loanMachine.connect(lender1).coverLoan(requisitionId, 70))
        .to.emit(loanMachine, "LoanFunded")
        .withArgs(requisitionId);

      const requisition = await loanMachine.getRequisitionInfo(requisitionId);
      expect(requisition.status).to.equal(3); // Active
      expect(requisition.currentCoverage).to.equal(70);
    });

    it("Should handle multiple lenders reaching minimum coverage", async function () {
      const tx = await loanMachine.connect(borrower).createLoanRequisition(
        toWei(2), 75, 30
      );
      const receipt = await tx.wait();
      const requisitionId = receipt.events.find(e => e.event === "LoanRequisitionCreated").args.requisitionId;

      // First lender covers 40%
      await loanMachine.connect(lender1).coverLoan(requisitionId, 40);
      
      // Second lender covers 35% (total 75% - minimum reached)
      await expect(loanMachine.connect(lender2).coverLoan(requisitionId, 35))
        .to.emit(loanMachine, "LoanFunded")
        .withArgs(requisitionId);

      const requisition = await loanMachine.getRequisitionInfo(requisitionId);
      expect(requisition.currentCoverage).to.equal(75);
    });
  });

  describe("Coverage Percentage Validation", function () {
    beforeEach(async function () {
      await loanMachine.connect(lender1).donate({ value: toWei(5) });
      await loanMachine.connect(borrower).createLoanRequisition(
        toWei(2), 80, 30
      );
    });

    it("Should revert when covering with 0%", async function () {
      await expect(loanMachine.connect(lender1).coverLoan(0, 0))
        .to.be.revertedWithCustomError(loanMachine, "InvalidCoveragePercentage");
    });

    it("Should revert when covering with 101%", async function () {
      await expect(loanMachine.connect(lender1).coverLoan(0, 101))
        .to.be.revertedWithCustomError(loanMachine, "InvalidCoveragePercentage");
    });

    it("Should accept coverage of 1%", async function () {
      const tx = await loanMachine.connect(borrower).createLoanRequisition(
        toWei(1), 80, 30
      );
      const receipt = await tx.wait();
      const requisitionId = receipt.events.find(e => e.event === "LoanRequisitionCreated").args.requisitionId;

      await expect(loanMachine.connect(lender1).coverLoan(requisitionId, 1))
        .to.emit(loanMachine, "LoanCovered");
    });

    it("Should accept coverage of 100%", async function () {
      const tx = await loanMachine.connect(borrower).createLoanRequisition(
        toWei(1), 80, 30
      );
      const receipt = await tx.wait();
      const requisitionId = receipt.events.find(e => e.event === "LoanRequisitionCreated").args.requisitionId;

      await expect(loanMachine.connect(lender1).coverLoan(requisitionId, 100))
        .to.emit(loanMachine, "LoanCovered");
    });
  });

  describe("Edge Cases for Percentage Limits", function () {
    beforeEach(async function () {
      await loanMachine.connect(lender1).donate({ value: toWei(10) });
    });

    it("Should handle exact minimum coverage requirement", async function () {
      // Test the boundary condition
      const tx = await loanMachine.connect(borrower).createLoanRequisition(
        toWei(1), 70, 30  // Exactly the minimum
      );
      const receipt = await tx.wait();
      const requisitionId = receipt.events.find(e => e.event === "LoanRequisitionCreated").args.requisitionId;

      await loanMachine.connect(lender1).coverLoan(requisitionId, 70);

      const requisition = await loanMachine.getRequisitionInfo(requisitionId);
      expect(requisition.status).to.equal(3); // Active
    });

    it("Should handle multiple small coverages adding up to minimum", async function () {
      const tx = await loanMachine.connect(borrower).createLoanRequisition(
        toWei(1), 70, 30
      );
      const receipt = await tx.wait();
      const requisitionId = receipt.events.find(e => e.event === "LoanRequisitionCreated").args.requisitionId;

      // Multiple small coverages
      await loanMachine.connect(lender1).coverLoan(requisitionId, 35);
      await loanMachine.connect(lender1).coverLoan(requisitionId, 35); // Total 70%

      const requisition = await loanMachine.getRequisitionInfo(requisitionId);
      expect(requisition.status).to.equal(3); // Active
    });
  });

  describe("Direct Borrowing", function () {
    beforeEach(async function () {
      await loanMachine.connect(lender1).donate({ value: toWei(10) });
    });

    it("Should allow direct borrowing", async function () {
      await loanMachine.connect(borrower).donate({ value: toWei(0.1) });
      await expect(loanMachine.connect(borrower).borrow(toWei(1)))
        .to.emit(loanMachine, "Borrowed");
    });

    it("Should revert if borrower has existing debt", async function () {
      await loanMachine.connect(borrower).donate({ value: toWei(0.1) });
      await loanMachine.connect(borrower).borrow(toWei(1));

      await expect(loanMachine.connect(borrower).borrow(toWei(1)))
        .to.be.revertedWithCustomError(loanMachine, "BorrowNotExpired");
    });
  });

  describe("Repayment System", function () {
    beforeEach(async function () {
      await loanMachine.connect(lender1).donate({ value: toWei(10) });
      await loanMachine.connect(borrower).donate({ value: toWei(0.1) });
      await loanMachine.connect(borrower).borrow(toWei(2));
    });

    it("Should allow repayment", async function () {
      await expect(loanMachine.connect(borrower).repay({ value: toWei(1) }))
        .to.emit(loanMachine, "Repaid");
    });

    it("Should revert when repaying without active borrowing", async function () {
      await expect(loanMachine.connect(lender1).repay({ value: toWei(1) }))
        .to.be.revertedWithCustomError(loanMachine, "NoActiveBorrowing");
    });
  });
});