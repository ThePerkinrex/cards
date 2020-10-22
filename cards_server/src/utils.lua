-- copy table
function deepcopy(orig)
    local orig_type = type(orig)
    local copy
    if orig_type == 'table' then
        copy = {}
        for orig_key, orig_value in next, orig, nil do
            copy[deepcopy(orig_key)] = deepcopy(orig_value)
        end
        setmetatable(copy, deepcopy(getmetatable(orig)))
    else -- number, string, boolean, etc
        copy = orig
    end
    return copy
end

-- add card to top
function add_card(pile, card)
    if pile.cards == nil then
        pile.cards = {}
    end
    table.insert(pile.cards, card)
end

-- pop card from top
function pop_card(pile)
    if pile.cards == nil then
        error('Pile is non existant') 
    end
    if #pile.cards == 0 then
        error('Pile is empty') 
    end
    local r = pile.cards[#pile.cards]
    pile.cards[#pile.cards] = nil

    return r
end

-- add card to bottom
function add_card_bottom(pile, card)
    if pile.cards == nil then
        pile.cards = {}
    end
    table.insert(pile.cards, 1, card)
end

-- pop card from bottom
function pop_card_bottom(pile)
    if pile.cards == nil then
        error('Pile is non existant') 
    end
    table.remove(pile.cards, 1)
end
