import * as anchor from "@coral-xyz/anchor";
import { web3, BN } from "@coral-xyz/anchor";
import { createAccounts, requireBNEq, requireCustomError, requireNativeError, takeTenToPower } from "./utils/utils";
import { expect } from "chai";
import { TestClient } from "./utils/testClient";
import { MARK_PRICE_PRECISION, MAXIMUM_MARGIN_RATIO, MINIMUM_MARGIN_RATIO, PEG_PRECISION, TEN } from "./constants/numericConstants";
import { OracleSource } from "./utils/types";

describe("clearing house: initialize_market", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    let testCli: TestClient;

    const marketIndex = new BN(0);
    const ammBaseAssetReserve = new BN(1000);
    const ammQuoteAssetReserve = new BN(1000);
    // 1 hour
    const ammPeriodicity = new BN(60 * 60);
    const ammPegMultiplier = PEG_PRECISION;
    const oracleSource = OracleSource.PYTH;
    const marginRatioInitial = 2000;
    const marginRatioPartial = 625;
    const marginRatioMaintenance = 500;
    // oracle
    let price = new BN(100 * web3.LAMPORTS_PER_SOL);
    let conf = new BN(1 * web3.LAMPORTS_PER_SOL);
    let exponent = -9;
    let emaPrice = new BN(90 * web3.LAMPORTS_PER_SOL);
    let emaConf = new BN(2 * web3.LAMPORTS_PER_SOL);

    before(async () => {
        testCli = await TestClient.create(provider, 2);
        // oracle
        await testCli.pythInitializePrice(price, conf, exponent, emaPrice, emaConf);
        // clearinghouse
        await testCli.initializeRelevantAccounts(9, true);
        await testCli.initialize(true);
    });

    it('Fail if signer not admin', async () => {
        const signer = testCli.getSignerByIndex(1);
        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammQuoteAssetReserve,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                marginRatioInitial,
                marginRatioPartial,
                marginRatioMaintenance,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'ConstraintHasOne'
        );
    });

    it('Fail if quote reserve != base reserve', async () => {
        const signer = testCli.getCurrentSigner();
        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammBaseAssetReserve.addn(1),
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                marginRatioInitial,
                marginRatioPartial,
                marginRatioMaintenance,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'InvalidInitialPeg'
        );
    });

    it('Fail if amm_base_asset_reserve * amm_quote_asset_reserve overflows', async () => {
        const n = new BN(2).pow(new BN(192 / 2));
        const signer = testCli.getCurrentSigner();
        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                n,
                n,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                marginRatioInitial,
                marginRatioPartial,
                marginRatioMaintenance,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'MathError'
        );
    });

    it('Fail with invalid margin ratio', async () => {
        const signer = testCli.getCurrentSigner();
        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammQuoteAssetReserve,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                MINIMUM_MARGIN_RATIO - 1,
                marginRatioPartial,
                marginRatioMaintenance,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'InvalidMarginRatio'
        );

        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammQuoteAssetReserve,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                MAXIMUM_MARGIN_RATIO + 1,
                marginRatioPartial,
                marginRatioMaintenance,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'InvalidMarginRatio'
        );

        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammQuoteAssetReserve,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                marginRatioInitial,
                MINIMUM_MARGIN_RATIO - 1,
                marginRatioMaintenance,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'InvalidMarginRatio'
        );

        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammQuoteAssetReserve,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                marginRatioInitial,
                MINIMUM_MARGIN_RATIO + 1,
                marginRatioMaintenance,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'InvalidMarginRatio'
        );

        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammQuoteAssetReserve,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                marginRatioInitial,
                marginRatioPartial,
                MINIMUM_MARGIN_RATIO - 1,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'InvalidMarginRatio'
        );

        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammQuoteAssetReserve,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                marginRatioInitial,
                marginRatioPartial,
                MAXIMUM_MARGIN_RATIO + 1,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'InvalidMarginRatio'
        );

        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammQuoteAssetReserve,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                marginRatioPartial - 1,
                marginRatioPartial,
                marginRatioMaintenance,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'InvalidMarginRatio'
        );

        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammQuoteAssetReserve,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                marginRatioInitial,
                marginRatioMaintenance - 1,
                marginRatioMaintenance,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'InvalidMarginRatio'
        );
    });

    it('Pass', async () => {
        await testCli.initializeMarket(
            marketIndex,
            ammBaseAssetReserve,
            ammQuoteAssetReserve,
            ammPeriodicity,
            ammPegMultiplier,
            oracleSource,
            marginRatioInitial,
            marginRatioPartial,
            marginRatioMaintenance,
        );

        const market = (await testCli.getMarkets()).markets[marketIndex.toNumber()];
        expect(market.initialized).eq(1);
        expect(market.marginRatioInitial).eq(marginRatioInitial);
        expect(market.marginRatioPartial).eq(marginRatioPartial);
        expect(market.marginRatioMaintenance).eq(marginRatioMaintenance);
        expect(market.amm.oracleSource).deep.eq(OracleSource.PYTH);
        requireBNEq(market.amm.baseAssetReserve, ammBaseAssetReserve);
        requireBNEq(market.amm.quoteAssetReserve, ammQuoteAssetReserve);
        requireBNEq(market.amm.sqrtK, ammBaseAssetReserve);
        requireBNEq(market.amm.pegMultiplier, ammPegMultiplier);
        requireBNEq(market.amm.lastMarkPriceTwap, MARK_PRICE_PRECISION);

        requireBNEq(market.amm.lastOraclePrice, MARK_PRICE_PRECISION.mul(price).div(takeTenToPower(Math.abs(exponent))));
        requireBNEq(market.amm.lastOraclePriceTwap, MARK_PRICE_PRECISION.mul(emaPrice).div(takeTenToPower(Math.abs(exponent))));
        requireBNEq(market.amm.fundingPeriod, ammPeriodicity);
        requireBNEq(market.amm.minimumBaseAssetTradeSize, takeTenToPower(7));
        requireBNEq(market.amm.mininumQuoteAssetTradeSize, takeTenToPower(7));
        requireBNEq(market.amm.lastFundingRateTs, market.amm.lastMarkPriceTwapTs);
        requireBNEq(market.amm.lastFundingRateTs, market.amm.lastOraclePriceTwapTs);
    });

    it('Fail if market account has been initialized', async () => {
        const signer = testCli.getCurrentSigner();
        await requireCustomError(
            testCli.clearingHouse.methods.initializeMarket(
                marketIndex,
                ammBaseAssetReserve,
                ammQuoteAssetReserve,
                ammPeriodicity,
                ammPegMultiplier,
                oracleSource,
                marginRatioInitial,
                marginRatioPartial,
                marginRatioMaintenance,
            ).accounts({
                admin: signer.publicKey,
                state: testCli.state,
                markets: testCli.markets,
                oracle: testCli.pythPriceFeed,
            } as any)
                .signers([signer])
                .rpc(),
            'MarketIndexAlreadyInitialized'
        );
    });
});