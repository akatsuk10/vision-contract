// tests/comprehensive_flow.test.ts

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Contracts } from "../target/types/contracts";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getAccount,
  getMint,
} from "@solana/spl-token";
import { expect } from "chai";

describe("🔬 Comprehensive Product Launch Flow Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Contracts as Program<Contracts>;

  // Test accounts
  const protocolAdmin = Keypair.generate();
  const productOwner = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const user3 = Keypair.generate();
  const unauthorizedUser = Keypair.generate();

  // Test constants
  const TOKEN_SUPPLY = 1_000_000;
  const IPO_SLOTS = 5;
  const INITIAL_DEPOSIT = 2 * LAMPORTS_PER_SOL;
  const LAUNCH_DATE = Math.floor(Date.now() / 1000) + 86400; // 24 hours from now

  // Program derived addresses - will be set in before hook
  let globalConfig: PublicKey;
  let product: PublicKey;
  let treasury: PublicKey;
  let poolAuthority: PublicKey;
  let tokenMint: Keypair;
  let tokenPool: PublicKey;
  let userBid1: PublicKey;
  let userBid2: PublicKey;
  let userBid3: PublicKey;

  const confirm = async (tx: string) => {
    const latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({ signature: tx, ...latestBlockhash }, "confirmed");
  };

  const logBalance = async (publicKey: PublicKey, name: string) => {
    const balance = await provider.connection.getBalance(publicKey);
    console.log(`    💰 ${name} Balance: ${balance / LAMPORTS_PER_SOL} SOL`);
  };

  const logAccountInfo = async (address: PublicKey, name: string) => {
    try {
      const accountInfo = await provider.connection.getAccountInfo(address);
      console.log(`    📋 ${name}:`);
      console.log(`       Address: ${address.toString()}`);
      console.log(`       Owner: ${accountInfo?.owner.toString() || 'Not found'}`);
      console.log(`       Lamports: ${accountInfo?.lamports || 0}`);
      console.log(`       Data Length: ${accountInfo?.data.length || 0} bytes`);
    } catch (error) {
      console.log(`    ❌ Error fetching ${name}: ${error}`);
    }
  };

  before("🔧 Derives PDAs & Airdrops", async () => {
    console.log("\n🌧️  SETUP PHASE: PDA DERIVATION & AIRDROPS");
    console.log("=" .repeat(50));

    // 1. Airdrop to all accounts
    console.log("💰 Requesting airdrops...");
    const airdropAmount = 10 * LAMPORTS_PER_SOL;
    const accounts = [
      { keypair: protocolAdmin, name: "Protocol Admin" },
      { keypair: productOwner, name: "Product Owner" },
      { keypair: user1, name: "User 1" },
      { keypair: user2, name: "User 2" },
      { keypair: user3, name: "User 3" },
      { keypair: unauthorizedUser, name: "Unauthorized User" },
    ];

    const airdrops = await Promise.all(
      accounts.map(({ keypair }) => 
        provider.connection.requestAirdrop(keypair.publicKey, airdropAmount)
      )
    );
    await Promise.all(airdrops.map(confirm));

    // 2. Derive all PDAs
    console.log("\n🔑 Deriving Program Derived Addresses...");
    
    // Global Config PDA
    [globalConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("global-config")],
      program.programId
    );
    console.log(`   Global Config: ${globalConfig.toString()}`);

    // Product PDA (uses product owner as seed)
    [product] = PublicKey.findProgramAddressSync(
      [Buffer.from("product"), productOwner.publicKey.toBuffer()],
      program.programId
    );
    console.log(`   Product: ${product.toString()}`);

    // Treasury PDA (uses product as seed)
    [treasury] = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury"), product.toBuffer()],
      program.programId
    );
    console.log(`   Treasury: ${treasury.toString()}`);

    // Pool Authority PDA (uses product as seed)
    [poolAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), product.toBuffer()],
      program.programId
    );
    console.log(`   Pool Authority: ${poolAuthority.toString()}`);

    // User Bid PDAs
    [userBid1] = PublicKey.findProgramAddressSync(
      [Buffer.from("bid"), product.toBuffer(), user1.publicKey.toBuffer()],
      program.programId
    );
    [userBid2] = PublicKey.findProgramAddressSync(
      [Buffer.from("bid"), product.toBuffer(), user2.publicKey.toBuffer()],
      program.programId
    );
    [userBid3] = PublicKey.findProgramAddressSync(
      [Buffer.from("bid"), product.toBuffer(), user3.publicKey.toBuffer()],
      program.programId
    );
    console.log(`   User Bid 1: ${userBid1.toString()}`);
    console.log(`   User Bid 2: ${userBid2.toString()}`);
    console.log(`   User Bid 3: ${userBid3.toString()}`);

    // 3. Create token mint and derive token pool
    tokenMint = Keypair.generate();
    tokenPool = getAssociatedTokenAddressSync(
      tokenMint.publicKey,
      poolAuthority,
      true
    );
    console.log(`   Token Mint: ${tokenMint.publicKey.toString()}`);
    console.log(`   Token Pool: ${tokenPool.toString()}`);

    // 4. Verify balances
    console.log("\n💰 Verifying airdrop balances...");
    const balances = await Promise.all(
      accounts.map(async ({ keypair, name }) => {
        const balance = await provider.connection.getBalance(keypair.publicKey);
        console.log(`   ${name}: ${balance / LAMPORTS_PER_SOL} SOL`);
        return balance;
      })
    );

    // Ensure all accounts have sufficient balance
    expect(balances.every(b => b >= 5 * LAMPORTS_PER_SOL)).to.be.true;

    console.log("\n✅ Setup completed successfully!");
    console.log(`   • ${accounts.length} accounts funded`);
    console.log(`   • ${7} PDAs derived`); // globalConfig, product, treasury, poolAuthority, 3 userBids
    console.log(`   • Token mint keypair generated`);
    console.log(`   • Token pool address derived`);
    console.log("");
  });

  describe("🌐 Protocol Initialization", () => {
    it("Should initialize protocol with correct admin", async () => {
      console.log("\n🔧 PROTOCOL INITIALIZATION");
      console.log("=" .repeat(50));

      console.log("🚀 Executing init_protocol...");
      const tx = await program.methods.initProtocol()
      .accountsPartial({
  globalConfig,
  signer: protocolAdmin.publicKey,
  systemProgram: SystemProgram.programId,
})

      .signers([protocolAdmin])
        .rpc();

      await confirm(tx);
      console.log(`✅ Transaction confirmed: ${tx}`);

      // Fetch and verify config
      const config = await program.account.globalConfig.fetch(globalConfig);
      
      console.log("\n📊 Global Config State:");
      console.log(`   Protocol Admin: ${config.protocolAdmin.toString()}`);
      console.log(`   Bump: ${config.bump}`);

      await logAccountInfo(globalConfig, "Global Config");
      await logBalance(protocolAdmin.publicKey, "Protocol Admin");

      expect(config.protocolAdmin.toString()).to.eq(protocolAdmin.publicKey.toString());
      expect(config.bump).to.be.a('number');
      
      console.log("✅ Protocol initialization test passed!");
    });
  });

  describe("🚀 Product Launch", () => {
    it("Should launch product with detailed token creation", async () => {
      console.log("\n🚀 PRODUCT LAUNCH");
      console.log("=" .repeat(50));

      console.log("💰 Pre-launch balances:");
      await logBalance(productOwner.publicKey, "Product Owner");

      const args = {
        name: "Demo Product",
        description: "A comprehensive test product",
        tokenSymbol: "DPROD",
        initialDeposit: new anchor.BN(INITIAL_DEPOSIT),
        ipoSlots: IPO_SLOTS,
        initialTokenSupply: new anchor.BN(TOKEN_SUPPLY),
        launchDate: new anchor.BN(LAUNCH_DATE),
      };

      console.log("\n📋 Launch Parameters:");
      console.log(`   Name: ${args.name}`);
      console.log(`   Token Symbol: ${args.tokenSymbol}`);
      console.log(`   Initial Deposit: ${args.initialDeposit.toNumber() / LAMPORTS_PER_SOL} SOL`);
      console.log(`   IPO Slots: ${args.ipoSlots}`);
      console.log(`   Token Supply: ${args.initialTokenSupply.toNumber()}`);

      console.log("\n🚀 Executing launch_product...");
      const tx = await program.methods.launchProduct(args)
        .accountsPartial({
          maker: productOwner.publicKey,
          product,
          treasury,
          tokenMint: tokenMint.publicKey,
          tokenPool,
          poolAuthority,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([productOwner, tokenMint])
        .rpc();

      await confirm(tx);
      console.log(`✅ Launch transaction confirmed: ${tx}`);

      // Verify product state
      const productState = await program.account.product.fetch(product);
      console.log("\n📊 Product State Verification:");
      console.log(`   Maker: ${productState.maker.toString()}`);
      console.log(`   Name: ${productState.name}`);
      console.log(`   Phase: ${Object.keys(productState.phase)[0]}`);
      console.log(`   IPO Slots: ${productState.ipoSlots}`);
      console.log(`   Token Supply: ${productState.totalTokenSupply.toNumber()}`);

      // Verify token setup
      const mintInfo = await getMint(provider.connection, tokenMint.publicKey);
      const poolAccount = await getAccount(provider.connection, tokenPool);
      
      console.log("\n🪙 Token Verification:");
      console.log(`   Mint Authority: ${mintInfo.mintAuthority?.toString()}`);
      console.log(`   Pool Balance: ${poolAccount.amount.toString()}`);
      console.log(`   Pool Owner: ${poolAccount.owner.toString()}`);

      // Assertions
      expect(productState.maker.toString()).to.eq(productOwner.publicKey.toString());
      expect(productState.name).to.eq(args.name);
      expect(productState.ipoSlots).to.eq(args.ipoSlots);
      expect(poolAccount.amount.toString()).to.eq(args.initialTokenSupply.toString());
      
      console.log("✅ Product launch test passed!");
    });
  });

  describe("📥 User Bidding", () => {
    it("Should allow multiple users to place bids", async () => {
      console.log("\n📥 USER BIDDING");
      console.log("=" .repeat(50));

      const bidTests = [
        {
          user: user1,
          userBid: userBid1,
          amount: new anchor.BN(0.5 * LAMPORTS_PER_SOL),
          slots: 1,
          name: "User 1"
        },
        {
          user: user2,
          userBid: userBid2,
          amount: new anchor.BN(1.2 * LAMPORTS_PER_SOL),
          slots: 2,
          name: "User 2"
        },
        {
          user: user3,
          userBid: userBid3,
          amount: new anchor.BN(2.5 * LAMPORTS_PER_SOL),
          slots: 3,
          name: "User 3"
        }
      ];

      for (const { user, userBid, amount, slots, name } of bidTests) {
        console.log(`\n🎯 ${name} placing bid...`);
        console.log(`   Amount: ${amount.toNumber() / LAMPORTS_PER_SOL} SOL`);
        console.log(`   Slots: ${slots}`);

        const tx = await program.methods
          .userBidProduct(amount, slots)
          .accountsPartial({
            user: user.publicKey,
            product,
            userBid,
            treasury,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();

        await confirm(tx);
        console.log(`   ✅ Bid confirmed: ${tx}`);

        // Verify bid state
        const bidState = await program.account.userBid.fetch(userBid);
        expect(bidState.amount.toNumber()).to.eq(amount.toNumber());
        expect(bidState.slotsRequested).to.eq(slots);
        expect(bidState.status.pending).to.not.be.undefined;
      }

      console.log("\n✅ All user bidding tests passed!");
    });
  });

  describe("✅ Bid Processing", () => {
    it("Should allow product owner to approve bids", async () => {
      console.log("\n✅ BID APPROVAL");
      console.log("=" .repeat(50));

      const bidsToApprove = [
        { userBid: userBid1, name: "User 1" },
        { userBid: userBid2, name: "User 2" }
      ];

      for (const { userBid, name } of bidsToApprove) {
        console.log(`\n🎯 Approving ${name} bid...`);
        
        const tx = await program.methods
          .approveBid()
          .accountsPartial({
            productOwner: productOwner.publicKey,
            product,
            userBid,
          })
          .signers([productOwner])
          .rpc();

        await confirm(tx);
        console.log(`   ✅ Approval confirmed: ${tx}`);

        // Verify approval
        const bidState = await program.account.userBid.fetch(userBid);
        expect(bidState.status.approved).to.not.be.undefined;
        expect(bidState.tokenAmount.toNumber()).to.be.greaterThan(0);
      }

      console.log("\n✅ Bid approval tests passed!");
    });

    it("Should allow product owner to reject bids", async () => {
      console.log("\n❌ BID REJECTION");
      console.log("=" .repeat(50));

      console.log("🎯 Rejecting User 3 bid...");
      
      const tx = await program.methods
        .rejectBid()
        .accountsPartial({
          productOwner: productOwner.publicKey,
          product,
          userBid: userBid3,
          treasury,
          userAccount: user3.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([productOwner])
        .rpc();

      await confirm(tx);
      console.log(`✅ Rejection confirmed: ${tx}`);

      // Verify rejection
      const bidState = await program.account.userBid.fetch(userBid3);
      expect(bidState.status.rejected).to.not.be.undefined;

      console.log("✅ Bid rejection test passed!");
    });
  });

  describe("📊 Final State Verification", () => {
    it("Should display comprehensive final state", async () => {
      console.log("\n📊 FINAL STATE SUMMARY");
      console.log("=" .repeat(50));

      // Get final states
      const productState = await program.account.product.fetch(product);
      const bid1State = await program.account.userBid.fetch(userBid1);
      const bid2State = await program.account.userBid.fetch(userBid2);
      const bid3State = await program.account.userBid.fetch(userBid3);

      console.log("\n📦 Product Final State:");
      console.log(`   Name: ${productState.name}`);
      console.log(`   Phase: ${Object.keys(productState.phase)[0]}`);
      console.log(`   Total Slots: ${productState.ipoSlots}`);
      console.log(`   Approved Bids: ${productState.approvedBids}`);

      console.log("\n📥 Final Bid States:");
      const bids = [
        { name: "User 1", state: bid1State },
        { name: "User 2", state: bid2State },
        { name: "User 3", state: bid3State }
      ];

      bids.forEach(({ name, state }) => {
        console.log(`   ${name}: ${Object.keys(state.status)[0]} - ${state.amount.toNumber() / LAMPORTS_PER_SOL} SOL`);
      });

      console.log("\n💰 Final Balances:");
      await logBalance(treasury, "Treasury");
      await logBalance(productOwner.publicKey, "Product Owner");

      console.log("\n🎯 Test Summary:");
      console.log(`   ✅ Protocol initialized`);
      console.log(`   ✅ Product launched`);
      console.log(`   ✅ ${bids.length} bids placed`);
      console.log(`   ✅ ${productState.approvedBids} bids approved`);
      console.log(`   ✅ ${bids.filter(b => Object.keys(b.state.status)[0] === 'rejected').length} bids rejected`);

      expect(productState.approvedBids).to.eq(2);
      console.log("\n🎉 All tests completed successfully!");
    });
  });
});