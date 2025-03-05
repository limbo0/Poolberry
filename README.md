## Searching triangular arb leveraging multi-threading.


- Use only `N` highest volume assets.
    - Highest volume assets should be dynamically updated according to the trading volume on chain.

- Sol should always be the starting value and the ending value.

- Spawn N - 1 threads.
    - Every thread will have an input of SOL and an ouput of N-1 possible outputs.
    - For all the N-1 above outputs, spawn N-2 threads.
    - For every thread now, use input as the above output and calculate new output for all possible N-2 assets.
        - When the thread reaches here, we don't calculate `chile -> child` or `child -> parent`
        - Now for all the outputs above, calculate the output for the starting asset.


## Documentation
- Subscribe to log_subscribe 
- Receiving (14,15) seconds delayed information from chainstack websocket.
- Helius websocket provide finalized commitment for log_subscribe.
    - Changed the websocket commitment to confirmed (few seconds delay).
- Gather all the accounts which are involved in the transaction.
- Filter the addresses which are obviously not token addresses
    - Create a list of pre-defined such addresses
    - While the token addresses are checked, if the address is not an token account, add it to the list of addresses.
    - 
- chainstack websocket does not provide finalized commitment for log_subscribe.


## Thoughts  
- Lol
- Why receiving transaction after 4 or 5 minutes.
