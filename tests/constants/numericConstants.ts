import { BN } from "@coral-xyz/anchor";

export const PEG_PRECISION = new BN(10 ** 3);
export const MARK_PRICE_PRECISION = new BN(10 ** 10);
export const MARGIN_PRECISION = 10000;
export const MAXIMUM_MARGIN_RATIO = MARGIN_PRECISION;
export const MINIMUM_MARGIN_RATIO = MARGIN_PRECISION / 50;

export const ZERO = new BN(0);
export const TEN = new BN(10);