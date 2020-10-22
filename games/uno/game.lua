-- require "deepcopy"

name = "UNO"
version = "1.0.0"
players = {2, 10}

function setup(players)
	local deck = deepcopy(Pile)
	deck.cards = shuffle({
		PlusFour, PlusFour, PlusFour, PlusFour,
		BlueZero
	})
	deck.face_down = true

	function deck:on_click(player_piles)
		add_card(player_piles[1], pop_card(self))
		return player_piles
	end

	local stack = deepcopy(Pile)
	local hand = deepcopy(Pile)
	
	return {deck, stack},{hand} -- piles / player piles
end

Pile = {face_down = false, cards = {}}

PlusFour = {image = "cards/plus4.png", kind = "+4", color = "any"}
BlueZero = {image = "cards/blue0.png", kind = "0", color = "blue"}