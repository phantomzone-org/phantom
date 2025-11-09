# OTC quote with encrypted trade intent

The example implements a OTC desk quote generation program for BTC/USD pair. The program produces encrypted quotes from encrypted trade intents received from the clients.

Executing quote generation on encrypted intents mitigates front-running risks which usually restricts clients from seeking quotes from only 1 or 2 OTC desks. Now the client can send their trade intent to many OTC desks without reveling the volume nor the direction. Once client receives encrypted quotes from multiple OTC desks, client decrypts the quotes, and selects the OTC desk that provided the best quote.

The quote generation algorithm is simple but reflects actual considerations OTC desks would make to provide a quote.

## Implementation
