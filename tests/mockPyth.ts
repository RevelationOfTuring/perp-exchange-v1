import * as anchor from "@coral-xyz/anchor";
import { web3, BN } from "@coral-xyz/anchor";
import { TestClient } from "./utils/testClient";
import { requireBNEq } from "./utils/utils";
import { expect } from "chai";

describe("mock pyth", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  let testCli: TestClient;

  let price = new BN(100 * web3.LAMPORTS_PER_SOL);
  let conf = new BN(1 * web3.LAMPORTS_PER_SOL);
  let exponent = -9;
  let emaPrice = new BN(90 * web3.LAMPORTS_PER_SOL);
  let emaConf = new BN(2 * web3.LAMPORTS_PER_SOL);

  before(async () => {
    testCli = await TestClient.create(provider, 1, false);
  });


  it("initialize_price", async () => {
    await testCli.pythInitializePrice(price, conf, exponent, emaPrice, emaConf);
    let priceFeed = await testCli.getPythPriceFeed();
    requireBNEq(price, priceFeed.priceMessage.price);
    requireBNEq(conf, priceFeed.priceMessage.conf);
    expect(exponent).eq(priceFeed.priceMessage.exponent);
    requireBNEq(emaPrice, priceFeed.priceMessage.emaPrice);
    requireBNEq(emaConf, priceFeed.priceMessage.emaConf);
  });

  it("set_price", async () => {
    price = price.addn(1);
    await testCli.pythSetPrice(price);
    let priceFeed = await testCli.getPythPriceFeed();
    requireBNEq(price, priceFeed.priceMessage.price);
    requireBNEq(conf, priceFeed.priceMessage.conf);
    expect(exponent).eq(priceFeed.priceMessage.exponent);
    requireBNEq(emaPrice, priceFeed.priceMessage.emaPrice);
    requireBNEq(emaConf, priceFeed.priceMessage.emaConf);
  });

  it("set_ema_price", async () => {
    emaPrice = emaPrice.addn(1);
    await testCli.pythSetEmaPrice(emaPrice);
    let priceFeed = await testCli.getPythPriceFeed();
    requireBNEq(price, priceFeed.priceMessage.price);
    requireBNEq(conf, priceFeed.priceMessage.conf);
    expect(exponent).eq(priceFeed.priceMessage.exponent);
    requireBNEq(emaPrice, priceFeed.priceMessage.emaPrice);
    requireBNEq(emaConf, priceFeed.priceMessage.emaConf);
  });
});
