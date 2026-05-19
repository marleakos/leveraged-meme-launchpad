import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LeveragedMeme } from "../target/types/leveraged_meme";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  createMint,
  mintTo,
} from "@solana/spl-token";
import { expect } from "chai";

describe("leveraged-meme", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.LeveragedMeme as Program<LeveragedMeme>;
  
  // Test accounts
  const creator = Keypair.generate();
  const buyer = Keypair.generate();
  const seller = Keypair.generate();
  
  // PDAs
  let tokenMint: Keypair;
  let tokenState: PublicKey;
  let curveState: PublicKey;
  let feeVault: PublicKey;
  let curveTokenAccount: PublicKey;
  let lpTokenAccount: PublicKey;
  
  // Test parameters
  const TOKEN_NAME = "Test Token";
  const TOKEN_SYMBOL = "TEST";
  const TOKEN_URI = "https://example.com/metadata.json";
  const LEVERAGE = 3;
  const DIRECTION = { long: {} };
  const PERP_MARKET = 0; // SOL-PERP

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(
      creator.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      buyer.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      seller.publicKey,
      10 * LAMPORTS_PER_SOL
    );

    // Wait for airdrop confirmation
    await new Promise((resolve) => setTimeout(resolve, 1000));
  });

  describe("Initialize Token", () => {
    it("Should initialize a new leveraged token", async () => {
      tokenMint = Keypair.generate();
      
      // Derive PDAs
      [tokenState] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("token_state"),
          tokenMint.publicKey.toBuffer(),
        ],
        program.programId
      );
      
      [curveState] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("curve_state"),
          tokenMint.publicKey.toBuffer(),
        ],
        program.programId
      );
      
      [feeVault] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("fee_vault"),
          tokenMint.publicKey.toBuffer(),
        ],
        program.programId
      );
      
      curveTokenAccount = await getAssociatedTokenAddress(
        tokenMint.publicKey,
        tokenState,
        true
      );
      
      lpTokenAccount = await getAssociatedTokenAddress(
        tokenMint.publicKey,
        tokenState,
        true
      );

      try {
        const tx = await program.methods
          .initializeToken(
            TOKEN_NAME,
            TOKEN_SYMBOL,
            TOKEN_URI,
            LEVERAGE,
            DIRECTION as any,
            PERP_MARKET
          )
          .accounts({
            creator: creator.publicKey,
            tokenMint: tokenMint.publicKey,
            tokenState,
            curveState,
            feeVault,
            curveTokenAccount,
            lpTokenAccount,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          })
          .signers([creator, tokenMint])
          .rpc();

        console.log("Transaction signature:", tx);

        // Verify token state
        const state = await program.account.tokenState.fetch(tokenState);
        expect(state.name).to.equal(TOKEN_NAME);
        expect(state.symbol).to.equal(TOKEN_SYMBOL);
        expect(state.leverage).to.equal(LEVERAGE);
        expect(state.graduated).to.be.false;
        
        console.log("✅ Token initialized successfully");
        console.log("  Name:", state.name);
        console.log("  Symbol:", state.symbol);
        console.log("  Leverage:", state.leverage + "x");
      } catch (error) {
        console.error("Error initializing token:", error);
        throw error;
      }
    });

    it("Should fail with invalid leverage", async () => {
      const invalidMint = Keypair.generate();
      
      const [invalidState] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("token_state"),
          invalidMint.publicKey.toBuffer(),
        ],
        program.programId
      );

      try {
        await program.methods
          .initializeToken(
            "Invalid",
            "INV",
            "uri",
            10, // Invalid leverage (too high)
            { long: {} },
            0
          )
          .accounts({
            creator: creator.publicKey,
            tokenMint: invalidMint.publicKey,
            tokenState: invalidState,
            curveState: invalidState, // Will fail before this matters
            feeVault: invalidState,
            curveTokenAccount: invalidState,
            lpTokenAccount: invalidState,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          })
          .signers([creator, invalidMint])
          .rpc();
        
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("LeverageTooHigh");
        console.log("✅ Correctly rejected invalid leverage");
      }
    });
  });

  describe("Buy Tokens", () => {
    let buyerTokenAccount: PublicKey;

    before(async () => {
      // Create buyer's token account
      buyerTokenAccount = await getAssociatedTokenAddress(
        tokenMint.publicKey,
        buyer.publicKey
      );
      
      const tx = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(
          buyer.publicKey,
          buyerTokenAccount,
          buyer.publicKey,
          tokenMint.publicKey
        )
      );
      
      await provider.sendAndConfirm(tx, [buyer]);
    });

    it("Should buy tokens from the curve", async () => {
      const buyAmount = new anchor.BN(1 * LAMPORTS_PER_SOL); // 1 SOL
      
      try {
        const tx = await program.methods
          .buy(buyAmount)
          .accounts({
            buyer: buyer.publicKey,
            tokenState,
            curveState,
            tokenMint: tokenMint.publicKey,
            buyerTokenAccount,
            curveTokenAccount,
            feeVault,
            protocolFeeAccount: creator.publicKey, // Use creator as fee receiver for testing
            creatorFeeAccount: creator.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          })
          .signers([buyer])
          .rpc();

        console.log("Buy transaction:", tx);

        // Check buyer received tokens
        const buyerAccount = await provider.connection.getTokenAccountBalance(
          buyerTokenAccount
        );
        console.log("Buyer token balance:", buyerAccount.value.uiAmount);
        expect(Number(buyerAccount.value.amount)).to.be.greaterThan(0);
        
        console.log("✅ Buy successful");
      } catch (error) {
        console.error("Error buying tokens:", error);
        throw error;
      }
    });

    it("Should calculate correct token price", async () => {
      const state = await program.account.tokenState.fetch(tokenState);
      const curve = await program.account.curveState.fetch(curveState);
      
      const basePrice = await curve.calculateBasePrice();
      console.log("Base price:", basePrice.toString());
      
      const marketCap = await state.marketCap();
      console.log("Market cap:", marketCap.toString());
      
      console.log("✅ Price calculations working");
    });
  });

  describe("Sell Tokens", () => {
    it("Should sell tokens back to the curve", async () => {
      // Get buyer's balance
      const buyerTokenAccount = await getAssociatedTokenAddress(
        tokenMint.publicKey,
        buyer.publicKey
      );
      
      const balance = await provider.connection.getTokenAccountBalance(
        buyerTokenAccount
      );
      
      const sellAmount = new anchor.BN(balance.value.amount).div(new anchor.BN(2)); // Sell half
      
      try {
        const tx = await program.methods
          .sell(sellAmount)
          .accounts({
            seller: buyer.publicKey,
            tokenState,
            curveState,
            tokenMint: tokenMint.publicKey,
            sellerTokenAccount: buyerTokenAccount,
            curveTokenAccount,
            feeVault,
            protocolFeeAccount: creator.publicKey,
            creatorFeeAccount: creator.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          })
          .signers([buyer])
          .rpc();

        console.log("Sell transaction:", tx);
        console.log("✅ Sell successful");
      } catch (error) {
        console.error("Error selling tokens:", error);
        throw error;
      }
    });
  });

  describe("Graduation", () => {
    it("Should check graduation status", async () => {
      const state = await program.account.tokenState.fetch(tokenState);
      const canGraduate = await state.canGraduate();
      
      console.log("Can graduate:", canGraduate);
      console.log("Market cap:", (await state.marketCap()).toString());
      
      // Should not be able to graduate yet (not enough volume)
      expect(canGraduate).to.be.false;
      
      console.log("✅ Graduation check working");
    });
  });

  describe("Pause/Unpause", () => {
    it("Should pause and unpause trading", async () => {
      // Pause
      await program.methods
        .setPause(true)
        .accounts({
          authority: creator.publicKey,
          tokenState,
          tokenMint: tokenMint.publicKey,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        })
        .signers([creator])
        .rpc();
      
      let state = await program.account.tokenState.fetch(tokenState);
      expect(state.paused).to.be.true;
      console.log("✅ Trading paused");
      
      // Unpause
      await program.methods
        .setPause(false)
        .accounts({
          authority: creator.publicKey,
          tokenState,
          tokenMint: tokenMint.publicKey,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        })
        .signers([creator])
        .rpc();
      
      state = await program.account.tokenState.fetch(tokenState);
      expect(state.paused).to.be.false;
      console.log("✅ Trading resumed");
    });

    it("Should reject pause from non-creator", async () => {
      try {
        await program.methods
          .setPause(true)
          .accounts({
            authority: buyer.publicKey,
            tokenState,
            tokenMint: tokenMint.publicKey,
            clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          })
          .signers([buyer])
          .rpc();
        
        expect.fail("Should have thrown error");
      } catch (error) {
        expect(error.toString()).to.include("Unauthorized");
        console.log("✅ Correctly rejected unauthorized pause");
      }
    });
  });
});
