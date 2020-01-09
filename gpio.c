#include <stdint.h>

#define BCM2708_PERI_BASE         0xFE000000
#define GPIO_BASE                (BCM2708_PERI_BASE + 0x200000) /* GPIO controller */

#include <fcntl.h>
#include <sys/mman.h>
#include <unistd.h>

#define PAGE_SIZE (4*1024)
#define BLOCK_SIZE (4*1024)

int  mem_fd;
void *gpio_map;

// I/O access
volatile unsigned *gpio;


// GPIO setup macros. Always use INP_GPIO(x) before using OUT_GPIO(x) or SET_GPIO_ALT(x,y)
#define INP_GPIO(g) *(gpio+((g)/10)) &= ~(7<<(((g)%10)*3))
#define OUT_GPIO(g) *(gpio+((g)/10)) |=  (1<<(((g)%10)*3))
#define SET_GPIO_ALT(g,a) *(gpio+(((g)/10))) |= (((a)<=3?(a)+4:(a)==4?3:2)<<(((g)%10)*3))

#define GPIO_SET *(gpio+7)  // sets   bits which are 1 ignores bits which are 0
#define GPIO_CLR *(gpio+10) // clears bits which are 1 ignores bits which are 0

#define GET_GPIO(g) (*(gpio+13)&(1<<g)) // 0 if LOW, (1<<g) if HIGH

#define GPIO_PULL *(gpio+37) // Pull up/pull down
#define GPIO_PULLCLK0 *(gpio+38) // Pull up/pull down clock

int32_t init()
{
   if ((mem_fd = open("/dev/mem", O_RDWR|O_SYNC) ) < 0) 
      return -1;

   /* mmap GPIO */
   gpio_map = mmap(
      NULL,             //Any adddress in our space will do
      BLOCK_SIZE,       //Map length
      PROT_READ|PROT_WRITE,// Enable reading & writting to mapped memory
      MAP_SHARED,       //Shared with other processes
      mem_fd,           //File to map
      GPIO_BASE         //Offset to GPIO peripheral
   );

   close(mem_fd); //No need to keep mem_fd open after mmap

   if (gpio_map == MAP_FAILED) 
   {
    return -1;
   }

   gpio = (volatile unsigned *)gpio_map;

   return 0;
}

void init_port(uint8_t port, uint8_t in_out)
{
    INP_GPIO(port); // must use INP_GPIO before we can use OUT_GPIO

    if(in_out)
    {
        OUT_GPIO(port);
    }
}

void set_port(uint8_t port, uint8_t state)
{
    if(state)
    {
        GPIO_SET = 1<<port;
    }
    else
    {
        GPIO_CLR = 1<<port;
    }
}

uint8_t get_port(uint8_t port)
{
    (void)port;
    //not needed for now
    return 0;
}