import { AnchorError, AnchorProvider, BN, Wallet, web3 } from "@coral-xyz/anchor";
import { expect } from "chai";
import { TEN } from "../constants/numericConstants";
type PublicKey = web3.PublicKey;

function requireBNEq(a: BN, b: BN) {
    expect(a.toString()).eq(b.toString());
}

function requirePublickeyEq(a: web3.PublicKey, b: web3.PublicKey) {
    expect(a.toBase58()).eq(b.toBase58());
}

async function requireCustomError(p: Promise<any>, errorCode: string, logged = false) {
    try {
        await p;
        throw new Error('No error thrown: Expect to throw error but none in fact');
    } catch (e) {
        if (e instanceof Error && e.message.startsWith('No error thrown')) {
            throw e;
        }

        let error = e as AnchorError;
        if (logged) {
            console.log(error);
        }

        expect(error.error.errorCode.code).eq(errorCode);
    }
}

async function requireNativeError(
    p: Promise<any>,
    expectedTransactionMessage: string,
    expectedTransactionLogsIndexes: Array<number> = [],
    expectedTransactionLogs: Array<string> = []
) {
    try {
        await p;
    } catch (e) {
        expect(expectedTransactionLogsIndexes.length).eq(expectedTransactionLogs.length);
        expect(e.transactionMessage).eq(expectedTransactionMessage);
        for (let i = 0; i < expectedTransactionLogsIndexes.length; i++) {
            expect(expectedTransactionLogs[i]).eq(e.transactionLogs[expectedTransactionLogsIndexes[i]]);
        }
    }
}

async function createAccounts(
    provider: AnchorProvider,
    spaces: Array<number>,
    programId: PublicKey,
): Promise<Array<PublicKey>> {
    const tx = new web3.Transaction();
    const wallet = provider.wallet as Wallet;
    const keys = [wallet.payer];
    for (const space of spaces) {
        const key = web3.Keypair.generate();
        keys.push(key);
        const rent = await provider.connection.getMinimumBalanceForRentExemption(space);
        tx.add(
            web3.SystemProgram.createAccount({
                fromPubkey: wallet.publicKey,
                newAccountPubkey: key.publicKey,
                lamports: rent,
                space,
                programId,
            })
        );
    }

    await web3.sendAndConfirmTransaction(
        provider.connection,
        tx,
        keys
    );

    return keys.slice(1).map(x => x.publicKey);
}

function getSeedFromNumber(n: number): Uint8Array {
    const buf = Buffer.alloc(32);
    buf.writeUInt8(n, 31);
    return new Uint8Array(buf);
}

function takeTenToPower(exponent: number): BN {
    return TEN.pow(new BN(exponent));
}

export { requireBNEq, requirePublickeyEq, createAccounts, requireNativeError, requireCustomError, getSeedFromNumber, takeTenToPower };