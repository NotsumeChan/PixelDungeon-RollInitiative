-- Script del mod de ejemplo (docs/mods.md). Las funciones van en el entorno global: usa un prefijo propio.
-- ctx = {hero = {name, class, level, hp, ht, gold}, npc = {id, name, disposition}, depth, flags}

-- Condición: se puede apostar con al menos `cost` de oro.
function herrero_puede_apostar(ctx, cost)
  return ctx.hero.gold >= cost
end

-- Efecto: un d6 contra el troll. 5 o 6 gana el triple; si no, pierde lo apostado.
function herrero_apuesta(ctx, cost)
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
