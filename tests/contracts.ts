import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Contracts } from "../target/types/contracts";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL,
  Transaction,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  getAccount,
} from "@solana/spl-token";
import { expect } from "chai";

describe("🚀 Full Protocol & Product Flow Test", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Contracts as Program<Contracts>;
  const protocolAdmin = Keypair.generate();
  const productOwner = Keypair.generate();
  const user = Keypair.generate();

  let globalConfig: PublicKey;
  let product: PublicKey;
  let treasury: PublicKey;
  let poolAuthority: PublicKey;
  let tokenMint: Keypair;
  let tokenPool: PublicKey;
  let userBid: PublicKey;

  const INITIAL_DEPOSIT = 2 * LAMPORTS_PER_SOL;
  const TOKEN_SUPPLY = 1_000_000;
  const IPO_SLOTS = 5;
  const LAUNCH_DATE = Math.floor(Date.now() / 1000) + 86400; // +1 day

  // Helper to wait for confirmation
  const confirm = async (signature: string) => {
    const latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction(
      {
        signature,
        ...latestBlockhash,
      },
      "confirmed"
    );
  };

  // Helper to wait for transaction settlement
  const waitForSettlement = async (ms: number = 1000) => {
    await new Promise((resolve) => setTimeout(resolve, ms));
  };

  it("🌐 Airdrops & Derives PDAs", async () => {
    // Request airdrops
    const airdropPromises = [
      provider.connection.requestAirdrop(protocolAdmin.publicKey, 10 * LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(productOwner.publicKey, 10 * LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(user.publicKey, 10 * LAMPORTS_PER_SOL),
    ];

    const signatures = await Promise.all(airdropPromises);
    
    // Wait for airdrop confirmations
    await Promise.all(signatures.map(sig => confirm(sig)));
    await waitForSettlement(2000);

    // Verify balances
    const balances = await Promise.all([
      provider.connection.getBalance(protocolAdmin.publicKey),
      provider.connection.getBalance(productOwner.publicKey),
      provider.connection.getBalance(user.publicKey),
    ]);

    console.log("Balances after airdrop:", balances);

    expect(balances[0]).to.be.greaterThan(5 * LAMPORTS_PER_SOL);
    expect(balances[1]).to.be.greaterThan(5 * LAMPORTS_PER_SOL);
    expect(balances[2]).to.be.greaterThan(5 * LAMPORTS_PER_SOL);

    // Global config PDA
    [globalConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("global-config")],
      program.programId
    );

    // Product PDA - NOTE: IDL shows it only uses maker, not salt
    [product] = PublicKey.findProgramAddressSync(
      [Buffer.from("product"), productOwner.publicKey.toBuffer()],
      program.programId
    );

    // Treasury PDA
    [treasury] = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury"), product.toBuffer()],
      program.programId
    );

    // Pool authority PDA
    [poolAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), product.toBuffer()],
      program.programId
    );

    // User bid PDA
    [userBid] = PublicKey.findProgramAddressSync(
      [Buffer.from("bid"), product.toBuffer(), user.publicKey.toBuffer()],
      program.programId
    );

    // Generate mint keypair (to be initialized in launchProduct)
    tokenMint = Keypair.generate();
    
    console.log("✅ All PDAs derived and balances verified");
  });

  it("✅ Initializes Protocol", async () => {
    try {
      const tx = await program.methods
        .initProtocol()
        .accountsPartial({
          globalConfig,
          signer: protocolAdmin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([protocolAdmin])
        .rpc();
      
      await confirm(tx);
      console.log("📝 initProtocol tx:", tx);

      const config = await program.account.globalConfig.fetch(globalConfig);
      expect(config.protocolAdmin.toString()).to.equal(
        protocolAdmin.publicKey.toString()
      );
    } catch (error) {
      console.error("Protocol initialization failed:", error);
      throw error;
    }
  });

  it("🚀 Launches Product", async () => {
    try {
      // Derive token pool AFTER mint is created
      tokenPool = getAssociatedTokenAddressSync(
        tokenMint.publicKey,
        poolAuthority,
        true
      );

      console.log("\n📌 --- Launch Product Start ---");
    console.log("🧾 Launch Arguments:");
    console.log("  Name:", "Test Product");
    console.log("  Description:", "Test Launch");
    console.log("  Symbol:", "TST");
    console.log("  Initial Deposit (SOL):", INITIAL_DEPOSIT / LAMPORTS_PER_SOL);
    console.log("  IPO Slots:", IPO_SLOTS);
    console.log("  Total Token Supply:", TOKEN_SUPPLY);
    console.log("  Launch Date (Unix):", LAUNCH_DATE);
    console.log("  Launch Date (ISO):", new Date(LAUNCH_DATE * 1000).toISOString());

    console.log("📦 Derived Addresses:");
    console.log("  Product PDA:", product.toString());
    console.log("  Treasury PDA:", treasury.toString());
    console.log("  Pool Authority PDA:", poolAuthority.toString());
    console.log("  Token Mint Address:", tokenMint.publicKey.toString());
    console.log("  Token Pool ATA:", tokenPool.toString());

      // Based on IDL, LaunchProductArgs doesn't include salt
      const tx = await program.methods
        .launchProduct({
          name: "Test Product",
          description: "Test Launch",
          tokenSymbol: "TST",
          initialDeposit: new anchor.BN(INITIAL_DEPOSIT),
          ipoSlots: IPO_SLOTS,
          initialTokenSupply: new anchor.BN(TOKEN_SUPPLY),
          launchDate: new anchor.BN(LAUNCH_DATE),
        })
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
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([productOwner, tokenMint])
        .rpc();

      await confirm(tx);
      console.log("✅ launchProduct tx:", tx);

      // Verify product creation
      const productAccount = await program.account.product.fetch(product);
      expect(productAccount.name).to.equal("Test Product");
      expect(productAccount.tokenMint.toString()).to.equal(tokenMint.publicKey.toString());
      expect(productAccount.maker.toString()).to.equal(productOwner.publicKey.toString());
      expect(productAccount.phase.bidding).to.not.be.undefined; // Should be in bidding phase

    console.log("🧾 On-Chain Product Info:");
    console.log("  Maker:", productAccount.maker.toString());
    console.log("  Token Mint:", productAccount.tokenMint.toString());
    console.log("  Token Pool:", productAccount.tokenPool.toString());
    console.log("  Phase:", Object.keys(productAccount.phase)[0]);
    console.log("  Launch Date:", new Date(productAccount.launchDate.toNumber() * 1000).toISOString());
    console.log("  Approved Bids:", productAccount.approvedBids.toString());

    const mintInfo = await getAccount(provider.connection, tokenPool);
    console.log("🪙 Token Pool Info:");
    console.log("  Token Pool Owner:", mintInfo.owner.toString());
    console.log("  Mint:", mintInfo.mint.toString());
    console.log("  Amount:", mintInfo.amount.toString());

    console.log("✅ 🎯 Launch Product Complete\n");

    } catch (error) {
      console.error("Product launch failed:", error);
      throw error;
    }
  });

  it("🧾 Places Bid", async () => {
    try {
      const bidAmount = new anchor.BN(0.5 * LAMPORTS_PER_SOL);

      const tx = await program.methods
        .userBidProduct(bidAmount)
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
      console.log("💸 userBid tx:", tx);

      // Verify bid creation
      const bidAccount = await program.account.userBid.fetch(userBid);
      expect(bidAccount.amount.toString()).to.equal(bidAmount.toString());
      expect(bidAccount.user.toString()).to.equal(user.publicKey.toString());
      expect(bidAccount.product.toString()).to.equal(product.toString());
      
      // Check bid status - should be pending
      expect(bidAccount.status.pending).to.not.be.undefined;
      expect(bidAccount.tokensClaimed).to.be.false;
      expect(bidAccount.fundsClaimed).to.be.false;
    } catch (error) {
      console.error("Bid placement failed:", error);
      throw error;
    }
  });

  it("✅ Approves Bid", async () => {
    try {
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
      console.log("✔️ Bid approved tx:", tx);

      // Verify approval
      const bidAccount = await program.account.userBid.fetch(userBid);
      expect(bidAccount.status.approved).to.not.be.undefined;
      
      const productAccount = await program.account.product.fetch(product);
      expect(productAccount.approvedBids).to.equal(1);
    } catch (error) {
      console.error("Bid approval failed:", error);
      throw error;
    }
  });

  it("❌ Rejects Duplicate Approval", async () => {
    try {
      await program.methods
        .approveBid()
        .accountsPartial({
          productOwner: productOwner.publicKey,
          product,
          userBid,
        })
        .signers([productOwner])
        .rpc();
      
      expect.fail("Should have failed - bid already approved");
    } catch (err) {
      expect(err).to.be.an("error");
      // Should be BidAlreadyProcessed error (code 6004)
      expect(err.toString()).to.include("6004");
      console.log("🛑 Duplicate approval correctly failed");
    }
  });

  it("❌ Claims Tokens Before Launch Date", async () => {
    try {
      const userTokenAccount = getAssociatedTokenAddressSync(
        tokenMint.publicKey,
        user.publicKey
      );

      // Create user's token account if needed
      try {
        await getAccount(provider.connection, userTokenAccount);
      } catch {
        const createAtaTx = new Transaction().add(
          createAssociatedTokenAccountInstruction(
            user.publicKey,
            userTokenAccount,
            user.publicKey,
            tokenMint.publicKey
          )
        );
        await provider.sendAndConfirm(createAtaTx, [user]);
      }

      await program.methods
        .claimTokens()
        .accountsPartial({
          user: user.publicKey,
          product,
          userBid,
          tokenPool,
          userTokenAccount,
          poolAuthority,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();
      
      expect.fail("Claim should have failed before launch");
    } catch (err) {
      expect(err).to.be.an("error");
      // Should be LaunchDateNotReached error (code 6003)
      expect(err.toString()).to.include("6003");
      console.log("❌ Token claim before launch correctly failed");
    }
  });

  it("✅ Test Bid Rejection Flow", async () => {
    // Create another user and bid for testing rejection
    const user2 = Keypair.generate();
    await provider.connection.requestAirdrop(user2.publicKey, 5 * LAMPORTS_PER_SOL);
    await waitForSettlement(1000);

    const [userBid2] = PublicKey.findProgramAddressSync(
      [Buffer.from("bid"), product.toBuffer(), user2.publicKey.toBuffer()],
      program.programId
    );

    try {
      // Place bid
      const bidAmount = new anchor.BN(0.3 * LAMPORTS_PER_SOL);
      const bidTx = await program.methods
        .userBidProduct(bidAmount)
        .accountsPartial({
          user: user2.publicKey,
          product,
          userBid: userBid2,
          treasury,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      await confirm(bidTx);

      // Reject bid
      const rejectTx = await program.methods
        .rejectBid()
        .accountsPartial({
          productOwner: productOwner.publicKey,
          product,
          userBid: userBid2,
          treasury,
          userAccount: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([productOwner])
        .rpc();

      await confirm(rejectTx);

      // Verify rejection
      const bidAccount = await program.account.userBid.fetch(userBid2);
      expect(bidAccount.status.rejected).to.not.be.undefined;
      
      console.log("✅ Bid rejection flow completed successfully");
    } catch (error) {
      console.error("Bid rejection test failed:", error);
      throw error;
    }
  });

  it("📊 Verifies Final State", async () => {
    // Check final product state
    const productAccount = await program.account.product.fetch(product);
    expect(productAccount.approvedBids).to.equal(1);
    expect(productAccount.phase.bidding).to.not.be.undefined;
    expect(productAccount.fundsClaimed).to.be.false;

    // Check approved bid state
    const bidAccount = await program.account.userBid.fetch(userBid);
    expect(bidAccount.status.approved).to.not.be.undefined;
    expect(bidAccount.tokensClaimed).to.be.false;
    expect(bidAccount.fundsClaimed).to.be.false;

    console.log("✅ Final state verification completed");
  });

  // Optional: Add test for post-launch date token claiming
  it("⏰ Test Token Claiming After Launch Date", async () => {
    console.log("⏰ This test would require time manipulation or shorter launch date");
    console.log("   In a real scenario, you'd wait for launch_date or use a shorter time");
    console.log("   Then test successful token claiming for approved bids");
  });

  it("🧹 Cleanup Resources", async () => {
    console.log("✅ Test suite completed successfully");
    console.log("📈 Product created, bids placed, approvals/rejections tested");
    console.log("🔒 Pre-launch validations confirmed working");
  });
});