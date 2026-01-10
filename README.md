# Which Option
Finance options calculator to inform you what strike and expiry to purchase

## How is it Different From Other Tools?
To be honest, it is not much different from other options tooling. However, the main layout and end goals are different from other tools.
 - Most other tools are designed for visualising payoff graphs given a portfolio of options. I.e payoff graphs and risk management are the main focuses.

The main goal of this tool is to answer the question: "Given a stock moves to a **given price in a given time duration**, **what option** should I purchase now to **maximise ROI?**".
 - Of course, no one knows for certainty how a stock will move, so this tool mainly designed to help with ~~my gambling addiction~~ a trader's speculative decisions
 - I.e maximising ROI for a specific scenario is the first priority. Payoff graphs and risk management come later.

This tool is also meant to be small, simple and straight to the point. (See down below for further details)

## Summary
Important details/notes/bugs about the tool:
 - All numerical environment data must be fed in manually
    - E.g there are no integrations to automatically pull in options pricing data
 - Option contract sizes are assumed to be 1
 - Black-Scholes pricing model is used
 - Prices are rounded to 2 d.p (nearest cent) in the direction that makes practical sense
    - When purchasing an option, prices are rounded up
    - When selling an option, prices are are rounded down
 - When calculating the optimal strike and expiry, gradient ascent is used on non-rounded calculations of Black-Scholes
    - Due to this, the "optimal" contract selected may not be the absolute highest point on the ROI payoff graphs (since the payoff graphs implement rounding)
 - IV is assumed to be constant across time and for all option strikes/expiries
    - This is probably the **biggest flaw of the tool**
    - I.e it pretends volatility smile doesn't exist

## Compiling From Source
Clone/download the repo and execute either of the following commands:
 - `cargo run` to compile and run a native version
 - `trunk serve` to compile and host a web version
