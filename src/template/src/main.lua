local text = "Hello World!"
local textObj = love.graphics.newText(love.graphics.getFont(), text)

local version = os.getenv("LOVER_PKG_VERSION")
local gameName = os.getenv("LOVER_PKG_NAME")

function love.draw()
    print(gameName, "v.", version)
    love.graphics.draw(textObj)
end