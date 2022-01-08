use crate::bindings::flash_bots_uniswap_query::FlashBotsUniswapQuery;
use ethers::prelude::*;

#[derive(Debug)]
pub struct CrossedPairManager<'a, M>
where
    M: Middleware,
{
    flash_query_contract: &'a FlashBotsUniswapQuery<M>,
    markets: Vec<TokenMarket<'a>>,
}

impl<'a, M> CrossedPairManager<'a, M>
where
    M: Middleware,
{
    pub fn new(
        grouped_pairs: &'a Vec<(H160, Vec<[H160; 3]>)>,
        flash_query_contract: &'a FlashBotsUniswapQuery<M>,
    ) -> Self {
        let pairs = grouped_pairs
            .into_iter()
            .map(|(token, pairs)| TokenMarket {
                token,
                pairs: pairs
                    .to_vec()
                    .into_iter()
                    .map(|[token0, token1, address]| Pair {
                        address,
                        token0,
                        token1,
                        reserve: None,
                    })
                    .collect::<Vec<Pair>>(),
            })
            .collect::<Vec<TokenMarket>>();
        Self {
            markets: pairs,
            flash_query_contract,
        }
    }

    pub async fn update_reserve(&mut self) {
        let reserves = self
            .get_all_pair_addresses()
            .iter()
            .map(|pair| pair.address)
            .collect::<Vec<H160>>();

        let reserves = self
            .flash_query_contract
            .get_reserves_by_pairs(reserves)
            .call()
            .await
            .unwrap();

        for (new_reserve, pair) in std::iter::zip(&reserves, self.get_all_pair_addresses()) {
            let updated_reserve = Reserve {
                reserve0: new_reserve[0],
                reserve1: new_reserve[1],
                block_timestamp_last: new_reserve[2],
            };

            pair.reserve = Some(updated_reserve);
        }
    }

    fn get_all_pair_addresses(&mut self) -> Vec<&mut Pair> {
        self.markets
            .iter_mut()
            .flat_map(|token_market| &mut token_market.pairs)
            .collect::<Vec<&mut Pair>>()
    }

    pub fn find_arbitrage_opportunities(&mut self) {
        for market in &mut self.markets {
            market.update_reserve_price();
            market.find_arbitrage_opportunity();
        }
        ()
    }
}

#[derive(Debug)]
pub struct TokenMarket<'a> {
    token: &'a H160,
    pairs: Vec<Pair>,
}

impl<'a> TokenMarket<'a> {
    pub fn update_reserve_price(&mut self) {
        for pair in &mut self.pairs {
            let reserve = pair.reserve.as_mut().unwrap();
        }
    }

    pub fn find_arbitrage_opportunity(&self) {
        for pair_a in &self.pairs {
            for pair_b in &self.pairs {
                let profit = pair_a
                    .reserve
                    .as_ref()
                    .unwrap()
                    .profit(pair_b.reserve.as_ref().unwrap());

                if profit.gt(&U256::from(10u128.pow(15))) {
                    dbg!(self.token);
                    dbg!(profit);
                    println!("------------------------------------------------------------------------------------------");
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Pair {
    address: H160,
    token0: H160,
    token1: H160,
    reserve: Option<Reserve>,
}

#[derive(Debug)]
pub struct Reserve {
    reserve0: U256,
    reserve1: U256,
    block_timestamp_last: U256,
}

impl Reserve {
    pub fn profit(&self, pair_b: &Self) -> U256 {
        // Uniswap return U112
        let divider = U256::from(10u128.pow(5));
        let divider_2 = U256::from(10u128.pow(10));
        let q = (self.reserve0 * pair_b.reserve1 / divider_2).as_u128() as f32;
        let r = (pair_b.reserve0 * self.reserve1 / divider_2).as_u128() as f32;
        let s = (self.reserve0 + pair_b.reserve0 / divider).as_u128() as f32;
        let x_opt = (r.powf(2.0f32) - ((r.powf(2.0f32) - q * r) / s)).powf(0.5f32) - r;
        let p = ((q * x_opt) / (r + s * x_opt) - x_opt) as u128 * 10u128.pow(5);

        U256::from(p)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn found_arbitrage() {}
}
