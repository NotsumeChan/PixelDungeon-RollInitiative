class User():
    __slots__ = ("icon", "id", "nickname", "email")
    def __init__(self) -> None:
        self.icon = None
        self.id : int = ""
        self.nickname : str = ""
        self.email : str = ""

    def LogIn(self) -> None:
        pass

    def SignUp(self) -> None:
        pass

    def LogOut(self) -> None:
        pass
    
    def SaveConfig(self,  ruta) -> None:
        pass

    def LoadConfig(self, archivo) -> None:
        pass