import * as anchor from "@coral-xyz/anchor";
import { web3 } from "@coral-xyz/anchor";
import { createAccounts, requireBNEq, requireCustomError, requirePublickeyEq } from "./utils/utils";
import { expect } from "chai";
import { TestClient } from "./utils/testClient";
import { ZERO } from "./constants/numericConstants";

describe("clearing house: initialize_history", () => {
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
            testCli.initializeHistory(),
            'ConstraintHasOne'
        );
        testCli.changeCurrentSigner(0);
    });

    it('Pass initialize history', async () => {
        let state = await testCli.getState();
        requirePublickeyEq(state.tradeHistory, web3.PublicKey.default);
        requirePublickeyEq(state.depositHistory, web3.PublicKey.default);
        requirePublickeyEq(state.fundingRateHistory, web3.PublicKey.default);
        requirePublickeyEq(state.fundingPaymentHistory, web3.PublicKey.default);
        requirePublickeyEq(state.liquidationHistory, web3.PublicKey.default);
        requirePublickeyEq(state.curveHistory, web3.PublicKey.default);

        await testCli.initializeHistory();
        state = await testCli.getState();
        requirePublickeyEq(state.tradeHistory, testCli.tradeHistory);
        requirePublickeyEq(state.depositHistory, testCli.depositHistory);
        requirePublickeyEq(state.fundingRateHistory, testCli.fundingRateHistory);
        requirePublickeyEq(state.fundingPaymentHistory, testCli.fundingPaymentHistory);
        requirePublickeyEq(state.liquidationHistory, testCli.liquidationHistory);
        requirePublickeyEq(state.curveHistory, testCli.curveHistory);

        const tradeHistory = await testCli.getTradeHistory();
        requireBNEq(tradeHistory.head, ZERO);
        expect(tradeHistory.tradeRecord.length).eq(1024);

        const fundingPaymentHistory = await testCli.getFundingPaymentHistory();
        requireBNEq(fundingPaymentHistory.head, ZERO);
        expect(fundingPaymentHistory.fundingPaymentRecords.length).eq(1024);

        const liquidationHistory = await testCli.getLiquidationHistory();
        requireBNEq(liquidationHistory.head, ZERO);
        expect(liquidationHistory.liquidationRecords.length).eq(1024);

        const depositHistory = await testCli.getDepositHistory();
        requireBNEq(depositHistory.head, ZERO);
        expect(depositHistory.depositRecords.length).eq(1024);

        const fundingRateHistory = await testCli.getFundingRateHistory();
        requireBNEq(fundingRateHistory.head, ZERO);
        expect(fundingRateHistory.fundingRateRecord.length).eq(1024);

        const curveHistory = await testCli.getCurveHistory();
        requireBNEq(curveHistory.head, ZERO);
        expect(curveHistory.curveRecords.length).eq(1024);
    });

    it('Fail if reinitialize', async () => {
        const [newTradeHistory, newDepositHistory, newLiquidationHistory, newFundingPaymentHistory, newFundingRateHistory, newCurveHistory] = await createAccounts(
            provider,
            [8 + 262160, 8 + 147472, 8 + 262160, 8 + 196624, 8 + 114704, 8 + 311312],
            testCli.clearingHouse.programId
        );
        const signer = testCli.getCurrentSigner();
        await requireCustomError(
            testCli.clearingHouse.methods.intializeHistory()
                .accounts({
                    admin: signer.publicKey,
                    state: testCli.state,
                    fundingPaymentHistory: newFundingPaymentHistory,
                    tradeHistory: newTradeHistory,
                    liquidationHistory: newLiquidationHistory,
                    depositHistory: newDepositHistory,
                    fundingRateHistory: newFundingRateHistory,
                    curveHistory: newCurveHistory,
                } as any)
                .signers([signer])
                .rpc(),
            'HistoriesAllInitialized'
        );
    });
});