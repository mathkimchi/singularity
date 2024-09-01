#!/usr/bin/env python3

"""
I'm trying to make it like a chat between this process and the main rust process.
Just for purposes of testing ipc.
"""

import tkinter

root = tkinter.Tk()
root.title("Subprocess UI")

chat_log = tkinter.Text(root)
chat_log.insert(tkinter.END, "Hi")

user_input = tkinter.Entry(root)

root.mainloop()
