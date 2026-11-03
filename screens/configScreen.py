#module imports
import pygame as py
from core import Screen, PickResolution, FrameRate

#screen define
configScreen = Screen(PickResolution[0], PickResolution[1])

#Buttons of the screen
ConfigButtons = {}

#Audio of the screen
ConfigAudio   = {}

#function
def ConfigSceen(Delta) -> None:
    configScreen.FrameRate(FrameRate/2)
    while configScreen.running:
        #get delta
        Delta = configScreen.deltaT(Delta)
        #clean screen
        configScreen.ResetScreen()
        #get keys pressed
        Keys = py.key.get_pressed()
        # Get the mouse position
        mouse_pos = py.mouse.get_pos()
        
        #event handler
        for event in py.event.get():
            configScreen.EventHandler(event, ConfigButtons)
        
        #window breaker
        if not configScreen.running:
            break

        # Draw a white rectangle at the mouse position
        py.draw.rect(configScreen.canvas, (255, 0, 0), (mouse_pos[0]-50, mouse_pos[1]-50, 100, 100))
        #screen updater
        configScreen.Update()
    configScreen.running = True