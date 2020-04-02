# We calculate the expected total number of collisions,
# from legitimate submissions: n - D + D((D-1)/D)^n
# n = # values, D = size of the output space
# source: http://matt.might.net/articles/counting-hash-collisions/
# To run this script install julia and run "julia collisions.jl".

bits = big"128"  # length of hash
ppl = 8000000000 # world population estimate
cens = 4*24*14   # CENs per person for two weeks, assuming rotatio frequency of 15 minutes
n = ppl*cens     # total number of CENs for two weeks
H = 2^bits       # size of hash output space, uniform distribution is assumed

print(n - H + H*((H-1)/H)^n)
