# The sharding proposal is as follows:
# Each shard corresponds to a time interval (e.g., 1-7 days) and a
# latitude / longitude interval (e.g., 0.25 degrees). This is chosen
# to be roughly city-sized (0.25 deg = 15 arcminutes = 15 nmi = 28 km
# along a line of longitude). The spatial blocks are narrower away
# from the equator, but at least 1/3 this size below 70 degrees
# latitude (beyond the arctic circle).

# This quantization is extremely easy to compute -- it involves
# rounding the latitude, longitude, and unix time -- so all users can
# determine which shards are relevant to them.

# The code below only implements spatially variable sharding,
# any timeframe can be assumed.
# Uncomment example at the bottom to print results,
# or play with shard_scaling.ipynb

# These are back of the envelope calculations, more care went into
# making sure they are correct, than making them pretty.

using Geodesy

## global constants
# report sizes
min_rep = 70
max_rep = 325
min_sig_rep = 134
max_sig_rep = 389

# conversion
MB = 1000000 # byte
GB = 1000000000 # byte

# coordinates
lat_min = -90
lat_max = 90
lon_min = -180
lon_max = 180

# world population
population = 8000000000 # ~2^33

# truncate mantissa of num to digits
function chop(num, digits)
    if num == 0.0 then
        return num
    else
        e = ceil(log10(abs(num)))
        scale = 10^(digits - e)
        return trunc(num * scale) / scale
    end
end

struct GeoParams
    pop_dens # estimated population/km^2 in dense areas
    inf_rate # estimated long tail infection rate per day, non-compounding
    rotations # how often the report keypair is rotated during the relevant period
end

# shards approximated as rectangles using cartesian distance
# curvature should be neglegible at these sizes
struct GeoShard
    # a, b, c, d are the vertices of the shard
    g::GeoParams
    a::LatLon
    b::LatLon
    c::LatLon
    d::LatLon
    side_a::Float64
    side_b::Float64
    area::Float64 # area in km^2
    pop_est::Float64 # estimated population per shard
    inf_est::Float64 # estimated infection per day
    up_est::Float64 # estimated report uploads
    down_est::Float64 # estimaded report downloads
    function GeoShard(g,a,b,c,d)
        side_a = distance(a,b)
        side_b = distance(a,d)
        area = (side_a*side_b)/1000000

        pop_est = area*g.pop_dens
        inf_est = pop_est*g.inf_rate

        up_est = inf_est*g.rotations
        down_est = up_est*pop_est

        return new(g,a,b,c,d,side_a,side_b,area,pop_est,inf_est,up_est,down_est)
    end
end

function compute_geo_bw(s::GeoShard)
    # report sizes in bytes
    dl_user_min = chop((s.up_est*min_rep)/MB, 3)
    dl_user_max = chop((s.up_est*max_sig_rep)/MB, 3)
    dl_all_min = chop((s.down_est*min_rep)/GB, 3)
    dl_all_max = chop((s.down_est*max_sig_rep)/GB, 3)
    # user_min/max in MB, all_min/max in GB
    return (dl_user_min, dl_user_max, dl_all_min, dl_all_max)
end

function show_GeoShard(s::GeoShard)
    #println("Area: ", chop(s.area, 3), " km^2")
    #println("Population: ", chop(s.pop_est, 3))
    #println("Infections per day: ", chop(s.inf_est, 3))
    println("Uploads: ", chop(s.up_est, 3))
    println("Downloads: ", chop(s.down_est, 3))
    umin, umax, amin, amax = compute_geo_bw(s)
    println("Download per user ", umin , " to ", umax, " MB")
    println("Total Download per day ", amin, " to ", amax, " GB")
    println()
end

function compute_all_GeoShards(g::GeoParams, step_angle)
    lats = lat_min:step_angle:lat_max
    lons = lon_min:step_angle:lon_max
    points = [ LatLon(x, y) for x in lats, y in lons]
    imax, jmax = size(points)
    shards = Matrix{Shard}(undef, imax,jmax)

    for i = 1:imax-1
        for j = 1:jmax-1
            A = view(points, i:i+1, j:j+1)
            shards[i,j] = Shard(g, A[1,1], A[1,2], A[2,2], A[2,1])
        end
    end
    return(shards)
end

function equator_shard(g::GeoParams, angle)
    s = GeoShard(g, LatLon(0, 0), LatLon(0, angle), LatLon(angle, 0), LatLon(angle, angle))
    return(s)
end

function large_shard(g::GeoParams)
    println("Shard at equator (large)")
    #   (GeoParams, a, b, c, d)
    s = GeoShard(g, LatLon(0,0), LatLon(0,0.25), LatLon(0.25,0), LatLon(0.25,0.25))
    return(s)
end

function small_shard(g::GeoParams)
    println("Shard at 70 deg (small)")
    s = GeoShard(g, LatLon(70.75,0), LatLon(70.75,0.25), LatLon(71,0), LatLon(71,0.25))
    return(s)
end

# calculate avg bandwidth per user in a shard on the equator
function bw(r, a)
    g = GeoParams(6000, 0.001, r)
    s = equator_shard(g, a)
    b1 ,b2 , _, _ = compute_geo_bw(s)
    b = (b1+b2)/2 #average between min and max report size
    return(b)
end

## Example:
# Population density 6000/km^2, 0.001 daily reporting rate
# Numbers for no, daily, hourly and quarter hourly rotations
# rotations = [1, 14, 336, 1344]
# gp = [ GeoParams(6000, 0.001, r) for r in rotations ]
# for g in gp
#     println("At ", g.rotations, " rotations")
#     show_GeoShard(large_shard(g))
# end
