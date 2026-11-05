-- Script del mod de ejemplo (docs/mods.md). Las funciones van en el entorno global: usa un prefijo propio.
-- ctx = {hero = {name, class, level, hp, ht, gold}, npc = {id, name, disposition}, depth, flags, settings}
-- ctx.settings["ejemplo-herrero"] = las opciones de mod.json (la pantalla de ajustes del mod).

local function opts(ctx)
  return (ctx.settings or {})["ejemplo-herrero"] or {}
end

-- Condición: se puede apostar con al menos `cost` de oro.
function herrero_puede_apostar(ctx, cost)
  local o = opts(ctx)
  if o.gamble == false then return false end
  return ctx.hero.gold >= (o.bet or cost)
end

-- Efecto: un d6 contra el troll. 5 o 6 gana el triple; si no, pierde lo apostado.
function herrero_apuesta(ctx, cost)
  cost = opts(ctx).bet or cost
  local d = math.random(6)
  if d >= 5 then
    return {
      {"gold", cost * 2},
      {"log", {es = "Sacas un " .. d .. ": el troll gruñe y te paga " .. cost * 3 .. " de oro.",
               en = "You roll a " .. d .. ": the troll grumbles and pays you " .. cost * 3 .. " gold."}},
    }
  end
  return {
    {"gold", -cost},
    {"log", {es = "Sacas un " .. d .. ": el troll se queda con tu oro.",
             en = "You roll a " .. d .. ": the troll keeps your gold."}},
  }
end
