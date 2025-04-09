import { AnchorProvider, web3, Program, IdlTypes } from "@coral-xyz/anchor";
import { createMint } from '@solana/spl-token';
import { createAccounts, getSeedFromNumber } from './utils';
import { ClearingHouse } from "../target/types/clearing_house";
type PublicKey = web3.PublicKey;

export class TestClient {
    provider: AnchorProvider;
    signers: Array<web3.Keypair>;
    currentSignerIndex: number;
    program: Program<ClearingHouse>;

    state: PublicKey;
    collateralMint: PublicKey;
    collateralVault: PublicKey;
    collateralVaultAuthority: PublicKey;
    insuranceVault: PublicKey;
    insuranceVaultAuthority: PublicKey;
    markets: PublicKey;

    fundingPaymentHistory: PublicKey;
    tradeHistory: PublicKey;
    liquidationHistory: PublicKey;
    depositHistory: PublicKey;
    fundingRateHistory: PublicKey;
    curveHistory: PublicKey;
    orderHistory: PublicKey;

    orderState: PublicKey;


    static async create(provider: AnchorProvider, clearingHouse: Program<ClearingHouse>, signersNum: number): Promise<TestClient> {
        const tc = new TestClient();
        tc.provider = provider;
        tc.program = clearingHouse;
        tc.signers = new Array<web3.Keypair>(signersNum);
        tc.currentSignerIndex = 0;
        for (let index = 0; index < signersNum; index++) {
            const key = web3.Keypair.fromSeed(getSeedFromNumber(index));
            tc.signers[index] = key;

            const tx = await tc.provider.connection.requestAirdrop(key.publicKey, 100 * web3.LAMPORTS_PER_SOL);
            await tc.provider.connection.confirmTransaction(tx);
            console.log(`signer${index} [${key.publicKey}]: ${await provider.connection.getBalance(key.publicKey)}`)
        }

        return tc;
    }

    async initializeHistoriesAccounts(logAddrs = false) {
        [this.tradeHistory, this.depositHistory, this.liquidationHistory, this.fundingPaymentHistory, this.fundingRateHistory, this.curveHistory] = await createAccounts(
            this.provider,
            [8 + 262160, 8 + 147472, 8 + 262160, 8 + 196624, 8 + 114704, 8 + 311312],
            this.program.programId
        );

        [this.orderHistory] = await createAccounts(
            this.provider,
            [8 + 458784],
            this.program.programId
        );

        if (logAddrs) {
            console.log(`tradeHistory: ${this.tradeHistory}
depositHistory: ${this.depositHistory}
liquidationHistory: ${this.liquidationHistory}
fundingPaymentHistory: ${this.fundingPaymentHistory}
fundingRateHistory: ${this.fundingRateHistory}
curveHistory: ${this.curveHistory}
orderHistory: ${this.orderHistory}`);
        }
    }

    async initializeRelevantAccounts(mintDecimal: number, logAddrs = false) {
        this.collateralMint = await this.createMint(mintDecimal);
        [this.collateralVault,] = web3.PublicKey.findProgramAddressSync([Buffer.from('collateral_vault')], this.program.programId);
        [this.collateralVaultAuthority,] = web3.PublicKey.findProgramAddressSync([this.collateralVault.toBuffer()], this.program.programId);
        [this.insuranceVault,] = web3.PublicKey.findProgramAddressSync([Buffer.from('insurance_vault')], this.program.programId);
        [this.insuranceVaultAuthority,] = web3.PublicKey.findProgramAddressSync([this.insuranceVault.toBuffer()], this.program.programId);
        [this.orderState,] = web3.PublicKey.findProgramAddressSync([Buffer.from('order_state')], this.program.programId);

        // create state && markets accounts
        [this.state, this.markets] = await createAccounts(
            this.provider,
            [8 + 1200, 8 + 31744],
            this.program.programId
        );

        if (logAddrs) {
            console.log(`collateral mint: ${this.collateralMint}
state: ${this.state}
collateral vault: ${this.collateralVault}
collateral vault authority: ${this.collateralVaultAuthority}
insurance vault: ${this.insuranceVault}
insurance vault authority: ${this.insuranceVaultAuthority}
markets: ${this.markets}
order state: ${this.orderState}`);
        }
    }

    async initializeOrderState() {
        const signer = this.getCurrentSigner();
        await this.program.methods.initializeOrderState()
            .accounts({
                admin: signer.publicKey,
                state: this.state,
                orderHistory: this.orderHistory
            } as any)
            .signers([signer])
            .rpc();
    }

    async initializeHistory() {
        const signer = this.getCurrentSigner();
        await this.program.methods.intializeHistory()
            .accounts({
                admin: signer.publicKey,
                state: this.state,
                fundingPaymentHistory: this.fundingPaymentHistory,
                tradeHistory: this.tradeHistory,
                liquidationHistory: this.liquidationHistory,
                depositHistory: this.depositHistory,
                fundingRateHistory: this.fundingRateHistory,
                curveHistory: this.curveHistory,
            } as any)
            .signers([signer])
            .rpc();
    }

    async initialize(adminControlsPrices: boolean) {
        const signer = this.getCurrentSigner();
        await this.program.methods.initialize(adminControlsPrices)
            .accounts({
                admin: signer.publicKey,
                state: this.state,
                collateralMint: this.collateralMint,
                collateralVaultAuthority: this.collateralVaultAuthority,
                insuranceVaultAuthority: this.insuranceVaultAuthority,
                markets: this.markets,
            })
            .signers([signer])
            .rpc();
    }

    async getState(): Promise<IdlTypes<ClearingHouse>['state']> {
        return await this.program.account.state.fetch(this.state);
    }

    async getMarkets(): Promise<IdlTypes<ClearingHouse>['markets']> {
        return await this.program.account.markets.fetch(this.markets);
    }

    async getFundingPaymentHistory(): Promise<IdlTypes<ClearingHouse>['fundingPaymentHistory']> {
        return await this.program.account.fundingPaymentHistory.fetch(this.fundingPaymentHistory);
    }

    async getTradeHistory(): Promise<IdlTypes<ClearingHouse>['tradeHistory']> {
        return await this.program.account.tradeHistory.fetch(this.tradeHistory);
    }

    async getLiquidationHistory(): Promise<IdlTypes<ClearingHouse>['liquidationHistory']> {
        return await this.program.account.liquidationHistory.fetch(this.liquidationHistory);
    }

    async getDepositHistory(): Promise<IdlTypes<ClearingHouse>['depositHistory']> {
        return await this.program.account.depositHistory.fetch(this.depositHistory);
    }

    async getFundingRateHistory(): Promise<IdlTypes<ClearingHouse>['fundingRateHistory']> {
        return await this.program.account.fundingRateHistory.fetch(this.fundingRateHistory);
    }

    async getCurveHistory(): Promise<IdlTypes<ClearingHouse>['curveHistory']> {
        return await this.program.account.curveHistory.fetch(this.curveHistory);
    }

    async getOrderHistory(): Promise<IdlTypes<ClearingHouse>['orderHistory']> {
        return await this.program.account.orderHistory.fetch(this.orderHistory);
    }

    async getOrderState(): Promise<IdlTypes<ClearingHouse>['orderState']> {
        return await this.program.account.orderState.fetch(this.orderState);
    }

    changeCurrentSigner(index: number) {
        this.currentSignerIndex = index;
    }

    getCurrentSigner(): web3.Keypair {
        return this.signers[this.currentSignerIndex];
    }

    async createMint(decimal: number): Promise<PublicKey> {
        const currentSigner = this.getCurrentSigner();
        return await createMint(
            this.provider.connection,
            currentSigner,
            currentSigner.publicKey,
            null,
            decimal);
    }
}