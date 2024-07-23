import { UmiPlugin } from '@metaplex-foundation/umi';
import { createMplxRewardsProgram } from './generated';

export const Rewards = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMplxRewardsProgram(), false);
  },
});