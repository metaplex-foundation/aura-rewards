/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  Account,
  Context,
  Pda,
  PublicKey,
  RpcAccount,
  RpcGetAccountOptions,
  RpcGetAccountsOptions,
  assertAccountExists,
  deserializeAccount,
  gpaBuilder,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  Serializer,
  array,
  publicKey as publicKeySerializer,
  struct,
  u128,
  u64,
  u8,
} from '@metaplex-foundation/umi/serializers';

export type RewardPool = Account<RewardPoolAccountData>;

export type RewardPoolAccountData = {
  depositAuthority: PublicKey;
  distributeAuthority: PublicKey;
  fillAuthority: PublicKey;
  rewardMint: PublicKey;
  indexWithPrecision: bigint;
  totalShare: bigint;
  distributionEndsAt: bigint;
  tokensAvailableForDistribution: bigint;
  tokenAccountBump: number;
  data: Array<number>;
};

export type RewardPoolAccountDataArgs = {
  depositAuthority: PublicKey;
  distributeAuthority: PublicKey;
  fillAuthority: PublicKey;
  rewardMint: PublicKey;
  indexWithPrecision: number | bigint;
  totalShare: number | bigint;
  distributionEndsAt: number | bigint;
  tokensAvailableForDistribution: number | bigint;
  tokenAccountBump: number;
  data: Array<number>;
};

export function getRewardPoolAccountDataSerializer(): Serializer<
  RewardPoolAccountDataArgs,
  RewardPoolAccountData
> {
  return struct<RewardPoolAccountData>(
    [
      ['depositAuthority', publicKeySerializer()],
      ['distributeAuthority', publicKeySerializer()],
      ['fillAuthority', publicKeySerializer()],
      ['rewardMint', publicKeySerializer()],
      ['indexWithPrecision', u128()],
      ['totalShare', u64()],
      ['distributionEndsAt', u64()],
      ['tokensAvailableForDistribution', u64()],
      ['tokenAccountBump', u8()],
      ['data', array(u8(), { size: 7 })],
    ],
    { description: 'RewardPoolAccountData' }
  ) as Serializer<RewardPoolAccountDataArgs, RewardPoolAccountData>;
}

export function deserializeRewardPool(rawAccount: RpcAccount): RewardPool {
  return deserializeAccount(rawAccount, getRewardPoolAccountDataSerializer());
}

export async function fetchRewardPool(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<RewardPool> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  assertAccountExists(maybeAccount, 'RewardPool');
  return deserializeRewardPool(maybeAccount);
}

export async function safeFetchRewardPool(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<RewardPool | null> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  return maybeAccount.exists ? deserializeRewardPool(maybeAccount) : null;
}

export async function fetchAllRewardPool(
  context: Pick<Context, 'rpc'>,
  publicKeys: Array<PublicKey | Pda>,
  options?: RpcGetAccountsOptions
): Promise<RewardPool[]> {
  const maybeAccounts = await context.rpc.getAccounts(
    publicKeys.map((key) => toPublicKey(key, false)),
    options
  );
  return maybeAccounts.map((maybeAccount) => {
    assertAccountExists(maybeAccount, 'RewardPool');
    return deserializeRewardPool(maybeAccount);
  });
}

export async function safeFetchAllRewardPool(
  context: Pick<Context, 'rpc'>,
  publicKeys: Array<PublicKey | Pda>,
  options?: RpcGetAccountsOptions
): Promise<RewardPool[]> {
  const maybeAccounts = await context.rpc.getAccounts(
    publicKeys.map((key) => toPublicKey(key, false)),
    options
  );
  return maybeAccounts
    .filter((maybeAccount) => maybeAccount.exists)
    .map((maybeAccount) => deserializeRewardPool(maybeAccount as RpcAccount));
}

export function getRewardPoolGpaBuilder(
  context: Pick<Context, 'rpc' | 'programs'>
) {
  const programId = context.programs.getPublicKey(
    'mplxRewards',
    'BF5PatmRTQDgEKoXR7iHRbkibEEi83nVM38cUKWzQcTR'
  );
  return gpaBuilder(context, programId)
    .registerFields<{
      depositAuthority: PublicKey;
      distributeAuthority: PublicKey;
      fillAuthority: PublicKey;
      rewardMint: PublicKey;
      indexWithPrecision: number | bigint;
      totalShare: number | bigint;
      distributionEndsAt: number | bigint;
      tokensAvailableForDistribution: number | bigint;
      tokenAccountBump: number;
      data: Array<number>;
    }>({
      depositAuthority: [0, publicKeySerializer()],
      distributeAuthority: [32, publicKeySerializer()],
      fillAuthority: [64, publicKeySerializer()],
      rewardMint: [96, publicKeySerializer()],
      indexWithPrecision: [128, u128()],
      totalShare: [144, u64()],
      distributionEndsAt: [152, u64()],
      tokensAvailableForDistribution: [160, u64()],
      tokenAccountBump: [168, u8()],
      data: [169, array(u8(), { size: 7 })],
    })
    .deserializeUsing<RewardPool>((account) => deserializeRewardPool(account));
}

export function getRewardPoolSize(): number {
  return 176;
}
