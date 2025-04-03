# no-no mouse

Ouch, ouch.. please, save my wrists.

Moving my "ergonomic" mouse is a big no-no.

It still hurts.

What about no-no mouse?

My keyboard is comfortable.

Wait. What if I controlled mouse's cursor+clicks+rolling
with my keyboard?

Ah, solved.

That is the NO-NO MOUSE!

## Usage

Aaaaah, I'll fill this later, good luck hahah

TODO: build, run

TODO: config

TODO: getting your keyboard device (sudo evtest)

### How to use really?

Somehow, like using your WM configuration, create a shortcut that will
execute no-no mouse.

Then you can activate no-no mouse mode with whatever key, whenever you want.

## Compatibility

It probably works on most Linux distros.

It **does not matter if you use X11 or Wayland**,
no-no mouse relies on `/dev/input` stuffs.

## Security

You have to use `sudo` to run no-no mouse.

Please, do a favor for yourself and read the code
before running it.

You want to save your wrists but you do not want to
risk hurting your computer.

> **Important: If you're using Ubuntu or any Linux distro with X11**,
> you might look for some other project that does not rely
> on `sudo`. See the [Similar projects](#Similar-projects) section.

## Contributing

Create an issue or a PR.

If you use AIs as code assistants, a good thing is to add both `./prompt.md`
and `./specs.md` to their context.

## License

MIT.

## Similar projects

- [philipl/evdevremapkeys](https://github.com/philipl/evdevremapkeys)
- [jordansissel/keynav](https://github.com/jordansissel/keynav) (only X11)
- If you have a full Gnome setup (e.g.: with Ubuntu), you can try
  their keyboard accessibility feature (not sure how complete is)

## TODOs

- [ ]: it does not need to press
- [ ]: Scrolling
  - [ ]: Page up, Page down?
- [ ]: Enable choosing mod key from config
- [ ]: Docs: Usage
- [ ]: Tests
- [ ]: build as a nix pkg
