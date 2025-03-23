local text = "Hello World!"
local textObj = love.graphics.newText(love.graphics.getFont(), text)

local version = os.getenv("LOVER_PKG_VERSION")
local gameName = os.getenv("LOVER_PKG_NAME")

print(gameName, "v.", version)

function love.draw()
    love.graphics.draw(textObj)
end