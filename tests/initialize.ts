import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import { ClearingHouse } from "../target/types/clearing_house";
import { createAccounts, requireCustomError, requireNativeError } from "./utils";
import { expect } from "chai";
import { TestClient } from "./testClient";

describe("clearing house: initialize", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.ClearingHouse as Program<ClearingHouse>;

    let testCli: TestClient;

    before(async () => {
        testCli = await TestClient.create(provider, program, 1);
        await testCli.initializeRelevantAccounts(9, true);
    });

    it('Fail with wrong collateral vault authority', async () => {
        const signer = testCli.getCurrentSigner();
        await requireCustomError(
            program.methods.initialize(true)
                .accounts({
                    admin: signer.publicKey,
                    state: testCli.state,
                    collateralMint: testCli.collateralMint,
                    collateralVaultAuthority: web3.Keypair.generate().publicKey,
                    insuranceVaultAuthority: testCli.insuranceVaultAuthority,
                    markets: testCli.markets
                })
                .signers([signer])
                .rpc(),
            'InvalidCollateralVaultAuthority'
        );
    });

    it('Fail with wrong insurance vault authority', async () => {
        const signer = testCli.getCurrentSigner();
        await requireCustomError(
            program.methods.initialize(true)
                .accounts({
                    admin: signer.publicKey,
                    state: testCli.state,
                    collateralMint: testCli.collateralMint,
                    collateralVaultAuthority: testCli.collateralVaultAuthority,
                    insuranceVaultAuthority: web3.Keypair.generate().publicKey,
                    markets: testCli.markets
                })
                .signers([signer])
                .rpc(),
            'InvalidInsuranceVaultAuthority'
        );
    });

    it('Pass initialize', async () => {
        await testCli.initialize(true);
        // check state
        const state = await testCli.getState();
        expect(state.adminControlsPrices).eq(1);
        const markets = await testCli.getMarkets();
        expect(markets.markets.length).eq(64);
    });

    it('Fail if initialize again with another state and markets', async () => {
        const [otherState, otherMarkets] = await createAccounts(
            provider,
            [8 + 1200, 8 + 31744],
            program.programId
        );

        const signer = testCli.getCurrentSigner();
        await requireNativeError(
            program.methods.initialize(true)
                .accounts({
                    admin: signer.publicKey,
                    state: otherState,
                    collateralMint: testCli.collateralMint,
                    collateralVaultAuthority: testCli.collateralVaultAuthority,
                    insuranceVaultAuthority: testCli.insuranceVaultAuthority,
                    markets: otherMarkets
                })
                .signers([signer])
                .rpc(),
            'Transaction simulation failed: Error processing Instruction 0: custom program error: 0x0',
            [3, 4],
            [
                `Allocate: account Address { address: ${testCli.collateralVault}, base: None } already in use`,
                'Program 11111111111111111111111111111111 failed: custom program error: 0x0'
            ]
        );
    });
});