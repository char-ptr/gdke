# gdke
A external and gui based version of [godot-key-extract](https://github.com/pozm/godot-key-extract)
# Images
<img width="912" alt="Screenshot 2024-09-03 at 07 18 29" src="https://github.com/user-attachments/assets/3c14010b-570e-4364-8baa-e4e5628152d1">
<img width="912" alt="Screenshot 2024-09-03 at 07 18 38" src="https://github.com/user-attachments/assets/f9dfb86f-5c40-4787-b81a-f61acaad74a9">
<img width="912" alt="Screenshot 2024-09-03 at 07 18 25" src="https://github.com/user-attachments/assets/bb226238-41d2-4277-8e08-9efb0caac30d">

### How does this work?
When you build a godot template with an encryption key set, the build tool (scons) will inline somewhere into the file. And so the key is in a random location pretty much every time you build.

We are still able to retrive this key though as it is obviously used to decrypt, encrypted scripts. and the place where it happens is in a function called `gdscript::load_byte_code`

![image](https://user-images.githubusercontent.com/44528100/211037537-f2b76cb7-2734-445a-a28d-c3bca404035d.png)

#### Finding statically
Thankfully it's really easy to find functions in ida, or any other modern static analysis program, as godot has verbose error logging. and we can abuse this to easily find the function.

![image](https://user-images.githubusercontent.com/44528100/211037616-76395bda-2fbf-43a5-81a9-a7da6374e0cb.png)

In ida, im able to go to where it is in rdata, and then find references as such:

![image](https://user-images.githubusercontent.com/44528100/211037662-501c041d-48e4-4813-9be7-bf4bead287df.png)

So now we've located the function which uses the secret key, all that's left to do is find where it's loaded (I recommend using graph view for next part). We can pretty easily find where it's loaded, although varies depending if the template was built in release or debug mode. Generally if it was built in release mode the key will be loaded near the beginning of the function, else in debug it will be right before it increments a for loop. We're looking for an instruction called `lea` (Load effective address) which takes a offset and loads it into a register. since our encryption key is pretty much static, it doesn't get passed in like a variable or what ever, it will always have a static offset. which makes it very easy to find. pretty much all the other `lea` instructions will load from a offset of a register.

If you have debug symbols it is extremely easy to find it as it will just be called `script_encryption_key`

![image](https://user-images.githubusercontent.com/44528100/211037804-c7270729-cdca-4f5d-8290-be613ef312c4.png)

If you do not have debug symbols it will be a bit harder to find, but still pretty trivial, it should look generally like:

![image](https://user-images.githubusercontent.com/44528100/211037865-16e58a09-74e8-43ae-a15c-fa27c123e6e7.png)

Once you have found the instruction, you should just be able to follow the offset, and read the bytes.
