use rand::distributions::Distribution;
use rand::distributions::Uniform;
use rand::Rng;

use crate::local_search::hill_climbing::HillClimbing;

use super::*;

pub use self::temperature::*;

pub trait Temperature<P>
   where P: Problem
{
   type State;
   fn acceptance_probability(&self, new: &Solution<P>, current: &Solution<P>, temp: &Self::State) -> f32;
   fn initial_temp(&self) -> Self::State;
   fn cool(&self, temp: &mut Self::State);
}

pub struct SimulatedAnnealing<I, N, R, A> {
   pub initial: I,
   pub neighborhood: N,
   pub create_random: R,
   pub heat: A,
}

impl<P, I, N, R, A, Rand> Algorithm<P> for SimulatedAnnealing<I, N, R, A>
   where P: Problem,
         I: InitialSolution<P>,
         N: Neighborhood<P>,
         <P as Problem>::Solution: Clone,
         <P as Problem>::Cost: Clone,
         R: Fn() -> Rand,
         A: Temperature<P>,
         Rand: Rng
{
   fn solve<'i, F>(&self, instance: &'i <P as Problem>::Instance, mut callback: F) -> Solution<'i, P>
                   where F: for<'a> FnMut(&'a Solution<P>) {
      let range = Uniform::new_inclusive(0.0, 1.0);

      let mut best = Solution::new(instance, self.initial.get(&instance));
      callback(&best);
      let mut current = best.clone();
      let mut rand = (&self.create_random)();
      let mut temp = self.heat.initial_temp();

      loop {
//         let cost = current.unwrap().cost().clone();
         match self.neighborhood
                   .find(current.clone(),
                         |s| s.cost() < current.cost() || self.heat.acceptance_probability(s, &current, &temp) >= range.sample(&mut rand)) {
            MaybeNeighbor::Found(s) => {
               current = s;
               if current.cost() < best.cost() {
                  best = current.clone();
                  callback(&best);
               }
               self.heat.cool(&mut temp);
            }
            MaybeNeighbor::NotFound(_) => break,
         }
      }

      /* assert that the best is really a local optimum */
      HillClimbing {
         initial: best.destroy().0,
         neighborhood: &self.neighborhood,
      }.solve(instance, callback)
//      best
   }
}

mod temperature {
   use num::cast::AsPrimitive;

   use super::*;

   pub struct DefaultTemperature {
      pub initial_temperature: f32,
      /// t' = t / cooling_factor
      pub cooling_factor: f32,
   }

   impl DefaultTemperature {
//      pub fn from_iterations() -> Self {
//         // TODO
//      }
   }

   impl<P> Temperature<P> for DefaultTemperature
      where P: Problem,
            <P as Problem>::Cost: AsPrimitive<f32>,
   {
      type State = f32;

      fn acceptance_probability<'a>(&self, new: &Solution<'a, P>, current: &Solution<'a, P>, temp: &f32) -> f32 {
         let new: f32 = new.cost().as_();
         let current: f32 = current.cost().as_();
         let delta = new - current + 1.0;
         let p = (-delta / temp).exp();
         p
      }

      fn initial_temp(&self) -> <Self as Temperature<P>>::State {
         self.initial_temperature
      }

      fn cool(&self, temp: &mut <Self as Temperature<P>>::State) {
         *temp /= self.cooling_factor;
      }
   }
}
