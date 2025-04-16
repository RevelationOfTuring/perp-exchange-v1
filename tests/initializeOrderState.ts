import * as anchor from "@coral-xyz/anchor";
import { web3, BN } from "@coral-xyz/anchor";
import { createAccounts, requireBNEq, requireCustomError, requireNativeError, requirePublickeyEq, ZERO_BN } from "./utils/utils";
import { expect } from "chai";
import { TestClient } from "./utils/testClient";

describe("clearing house: initialize_order_state", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    let testCli: TestClient;

    before(async () => {
        testCli = await TestClient.create(provider, 2, true, false);
        await testCli.initializeRelevantAccounts(9, true);
        await testCli.initializeHistoriesAccounts(true);
        await testCli.initialize(true);
    });

    it('Fail if signer not admin in state', async () => {
        testCli.changeCurrentSigner(1);
        await requireCustomError(
            testCli.initializeOrderState(),
            'ConstraintHasOne'
        );
        testCli.changeCurrentSigner(0);
    });

    it('Pass initialize order state', async () => {
        let state = await testCli.getState();
        requirePublickeyEq(state.orderState, web3.PublicKey.default);

        await testCli.initializeOrderState();
        state = await testCli.getState();
        requirePublickeyEq(state.orderState, testCli.orderState);

        const orderState = await testCli.getOrderState();
        requirePublickeyEq(orderState.orderHistory, testCli.orderHistory);
        requireBNEq(orderState.orderFillerRewardStructure.rewardNumerator, new BN(1));
        requireBNEq(orderState.orderFillerRewardStructure.rewardDenominator, new BN(10));
        requireBNEq(orderState.orderFillerRewardStructure.timeBasedRewardLowerBound, new BN(10000));
        requireBNEq(orderState.minOrderQuoteAssetAmount, new BN(500000));

        const orderHistory = await testCli.getOrderHistory();
        requireBNEq(orderHistory.head, ZERO_BN);
        requireBNEq(orderHistory.lastOrderId, ZERO_BN);
        expect(orderHistory.orderRecords.length).eq(1024);
    });

    it('Fail if reinitialize', async () => {
        const [newOrderHistory] = await createAccounts(
            provider,
            [8 + 458784],
            testCli.clearingHouse.programId
        );
        const signer = testCli.getCurrentSigner();

        await requireNativeError(
            testCli.clearingHouse.methods.initializeOrderState()
                .accounts({
                    admin: signer.publicKey,
                    state: testCli.state,
                    orderHistory: newOrderHistory
                } as any)
                .signers([signer])
                .rpc(),
            'Transaction simulation failed: Error processing Instruction 0: custom program error: 0x0',
            [3, 4],
            [
                `Allocate: account Address { address: ${testCli.orderState}, base: None } already in use`,
                'Program 11111111111111111111111111111111 failed: custom program error: 0x0'
            ]
        );
    });
});