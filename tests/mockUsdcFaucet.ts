import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
import { MockUsdcFaucet } from "../target/types/mock_usdc_faucet";
import { createAssociatedTokenAccount, createMint, getAccount, getMint, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { TestClient } from "./utils/testClient";
import { requirePublickeyEq } from "./utils/utils";
import { expect } from "chai";
type PublicKey = web3.PublicKey;

describe("mock usdc faucet", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MockUsdcFaucet as Program<MockUsdcFaucet>;

  let testCli: TestClient;
  let mockUsdcMint: PublicKey;
  let mintAuthorityPda;
  let mintAuthorityPdaBump;
  let state;


  before(async () => {
    testCli = await TestClient.create(provider, 2, false, false);
    const mockUsdcMintKey = web3.Keypair.generate();

    [mintAuthorityPda, mintAuthorityPdaBump] = web3.PublicKey.findProgramAddressSync([mockUsdcMintKey.publicKey.toBytes()], program.programId);
    console.log(`mintAuthorityPda: ${mintAuthorityPda}
mintAuthorityPdaBump: ${mintAuthorityPdaBump}`);

    [state,] = web3.PublicKey.findProgramAddressSync([Buffer.from('state')], program.programId);

    mockUsdcMint = await createMint(
      provider.connection,
      testCli.signers[0],
      mintAuthorityPda,
      null,
      9,
      mockUsdcMintKey,
      null,
      TOKEN_PROGRAM_ID
    );
    console.log(`mockUsdcMint: ${mockUsdcMint}`);
  });


  it("initialize_mock_usdc_faucet", async () => {
    await program.methods.initializeMockUsdcFaucet()
      .accounts({
        signer: testCli.signers[0].publicKey,
        usdcMint: mockUsdcMint,
      })
      .signers([testCli.signers[0]])
      .rpc();

    const stateAcc = await program.account.state.fetch(state);
    requirePublickeyEq(stateAcc.mint, mockUsdcMint);
    requirePublickeyEq(stateAcc.mintAuthorityPda, mintAuthorityPda);
    expect(stateAcc.mintAuthorityPdaBump).eq(mintAuthorityPdaBump);

    try {
      await program.methods.initializeMockUsdcFaucet()
        .accounts({
          signer: testCli.signers[0].publicKey,
          usdcMint: mockUsdcMint,
        })
        .signers([testCli.signers[0]])
        .rpc()
    } catch (error) {
      expect(error.transactionMessage).eq(`Transaction simulation failed: Error processing Instruction 0: custom program error: 0x0`)
      expect(error.transactionLogs[3]).eq(`Allocate: account Address { address: ${state}, base: None } already in use`)
    }

  });

  it("mint_to_user", async () => {
    // mint to signers[1]
    let amount = new BN(100);
    const ata1 = await createAssociatedTokenAccount(
      provider.connection,
      testCli.signers[1],
      mockUsdcMint,
      testCli.signers[1].publicKey,
    );

    await program.methods.mintToUser(amount)
      .accounts({
        reciever: ata1,
        state,
        mockUsdcMint,
        mintAuthorityPda
      })
      .rpc();

    let ataAcc = await getAccount(
      provider.connection,
      ata1
    );
    expect(ataAcc.amount).eq(BigInt(100));

    let mintAcc = await getMint(
      provider.connection,
      mockUsdcMint
    );
    expect(mintAcc.supply).eq(BigInt(0 + 100));

    // mint to signers[0]
    amount = new BN(100);
    const ata2 = await createAssociatedTokenAccount(
      provider.connection,
      testCli.signers[0],
      mockUsdcMint,
      testCli.signers[0].publicKey,
    );

    await program.methods.mintToUser(amount)
      .accounts({
        reciever: ata2,
        state,
        mockUsdcMint,
        mintAuthorityPda
      })
      .rpc();

    ataAcc = await getAccount(
      provider.connection,
      ata2
    );
    expect(ataAcc.amount).eq(BigInt(100));

    mintAcc = await getMint(
      provider.connection,
      mockUsdcMint
    );
    expect(mintAcc.supply).eq(BigInt(100 + 100));
  });
});
