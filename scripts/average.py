# Parse the output from `checkers-redux` to calculate average game statistics.
#
# Usage:
#
#   python3 scripts/average.py < output.txt
#
import fileinput

data: dict = {}


def add_value(d, keys, value):
    if len(keys) == 0:
        raise NotImplementedError
    if len(keys) == 1:
        d[keys[0]] = value
        return
    if keys[0] not in d.keys():
        d[keys[0]] = {}
    add_value(d[keys[0]], keys[1:], value)


for line in fileinput.input():
    keys, value = line.strip().split(" = ")
    add_value(data, keys.split("."), value)

stats = {
    "player1": {
        "wins": 0,
        "draws": 0,
        "losses": 0,
        "moves": 0,
        "explored": 0,
        "beta_cuts": 0,
        "tt_exact": 0,
        "tt_cuts": 0,
        "max_depth": 0,
    },
    "player2": {
        "wins": 0,
        "draws": 0,
        "losses": 0,
        "moves": 0,
        "explored": 0,
        "beta_cuts": 0,
        "tt_exact": 0,
        "tt_cuts": 0,
        "max_depth": 0,
    },
}

total_games = 0


def update_player_stats(stats, game, player):
    stats[player]["moves"] += int(game[player]["moves"])
    stats[player]["explored"] += int(game[player]["explored"])
    stats[player]["beta_cuts"] += int(game[player]["beta_cuts"])
    stats[player]["tt_exact"] += int(game[player]["tt_exact"])
    stats[player]["tt_cuts"] += int(game[player]["tt_cuts"])
    stats[player]["max_depth"] += int(game[player]["max_depth"])


for gid in data["game"].keys():
    total_games += 1
    game = data["game"][gid]
    if game["winner"] == "draw":
        stats["player1"]["draws"] += 1
        stats["player2"]["draws"] += 1
    elif game["winner"] == "player1":
        stats["player1"]["wins"] += 1
        stats["player2"]["losses"] += 1
    elif game["winner"] == "player2":
        stats["player2"]["wins"] += 1
        stats["player1"]["losses"] += 1
    update_player_stats(stats, game, "player1")
    update_player_stats(stats, game, "player2")


def average_player_stats(stats, total_games, player):
    stats[player]["moves"] = stats[player]["moves"] / total_games
    stats[player]["explored"] = stats[player]["explored"] / total_games
    stats[player]["beta_cuts"] = stats[player]["beta_cuts"] / total_games
    stats[player]["tt_exact"] = stats[player]["tt_exact"] / total_games
    stats[player]["tt_cuts"] = stats[player]["tt_cuts"] / total_games
    stats[player]["max_depth"] = stats[player]["max_depth"] / total_games


average_player_stats(stats, total_games, "player1")
average_player_stats(stats, total_games, "player2")

def print_config_player(player, config):
    for key in config[player].keys():
        print(player, key, "=", config[player][key])

def print_config(config):
    print("---- config")
    print("games = ", config["games"])
    print()
    print_config_player("player1", config)
    print()
    print_config_player("player2", config)

print()
print_config(data["config"])
print()

for player in stats.keys():
    print("==== [", player, "]")
    print()
    for val in stats[player].keys():
        print(val, "=", stats[player][val])
    print()
