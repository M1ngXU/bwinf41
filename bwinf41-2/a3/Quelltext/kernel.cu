#include <iostream>
#include <stdarg.h>

using u8 = unsigned char;
using u64 = unsigned long long;
struct Bestes
{
    u8 laenge;
    u8 letzter_flip;
};
struct BestesMitEnumeration
{
    u64 enumeration;
    Bestes bestes;
};

template <typename... Args>
inline __device__ void print(const char *f, Args... args)
{
    // printf(f, args...);
}

inline __device__ u64 enumerate_permutation(u8 *permutation, u8 *indeces, u8 len)
{
    print("Permutation: ");
    for (u8 i = 0; i < len; i++)
        print("%u|", permutation[i]);
    print("\n");
    for (u8 i = 0; i < len; i++)
    {
        u8 index = (u8)-1;
        for (u8 j = 0; j < len - i; j++)
        {
            if (index != (u8)-1)
                permutation[j - 1] = permutation[j];
            else if (permutation[j] - 1 == i)
                index = j;
        }
        assert(index != (u8)-1);
        indeces[i] = index;
    }
    print("Indeces: ");
    for (u8 i = 0; i < len; i++)
        print("%u|", indeces[i]);
    print("\n");
    u64 result = 0;
    u64 fact = 1;
    for (u8 i = 0; i < len; i++)
    {
        if (i > 1)
            fact *= i;
        result += indeces[len - 1 - i] * fact;
    }
    print("Enumeration: %u\n", result);
    return result;
}

inline __device__ void permutation_by_enumeration(u64 enumeration, u8 *result, u8 *indeces, u8 len, u64 fact)
{
    for (u8 i = 0; i < len; i++)
    {
        fact /= len - i;
        indeces[i] = enumeration / fact;
        enumeration %= fact;
    }
    for (u8 i = len - 1; i < len; i--)
    {
        u8 index = indeces[i];
        for (u8 j = len - i - 1; j > index; j--)
            result[j] = result[j - 1];
        result[index] = i + 1;
    }
}

inline __device__ void wenden_und_essen(u8 *stapel, u8 *neuer_stapel, u8 len, u8 index)
{
    print("Vorher: ");
    for (u8 i = 0; i < len; i++)
        print("%u|", stapel[i]);
    print("\n");
    u8 gegessen = stapel[index];
    for (u8 i = 0; i < index; i++)
    {
        u8 pfannkuchen = stapel[i];
        if (pfannkuchen > gegessen)
            pfannkuchen--;
        neuer_stapel[i] = pfannkuchen;
    }
    for (u8 i = 0; i < len - index - 1; i++)
    {
        u8 pfannkuchen = stapel[len - 1 - i];
        if (pfannkuchen > gegessen)
            pfannkuchen--;
        neuer_stapel[index + i] = pfannkuchen;
    }
    print("Nachher: ");
    for (u8 i = 0; i < len - 1; i++)
        print("%u|", neuer_stapel[i]);
    print("\n");
}

// https://en.wikipedia.org/wiki/Permutation#Generation_in_lexicographic_order
inline __device__ void permutate(u8 *stapel, u8 len, u64 enumeration)
{
    u8 k = len - 2;
    while (k <= len - 2 && stapel[k] > stapel[k + 1])
        k--;
    if (k == (u8)-1)
    {
        print("%llu|%u|", enumeration, len);
        for (u8 i = 0; i < len; i++)
            print("%u|", stapel[i]);
        print("\n");
        assert(false);
    }

    u8 i = len - 1;
    while (stapel[k] > stapel[i])
        i--;

    u8 tmp = stapel[k];
    stapel[k] = stapel[i];
    stapel[i] = tmp;

    u8 swap_count = (len - k) / 2;
    for (i = 0; i < swap_count; i++)
    {
        u8 tmp = stapel[k + 1 + i];
        stapel[k + 1 + i] = stapel[len - 1 - i];
        stapel[len - 1 - i] = tmp;
    }
}

extern "C" __global__ void run_permutations(Bestes *prior, Bestes *current, BestesMitEnumeration *bestes_gefundene, u8 size, u64 fact)
{
    u64 max_elements = fact / (blockDim.x * gridDim.x) + 1;
    u64 index = blockIdx.x * blockDim.x + threadIdx.x;
    u64 enumeration = index * max_elements;
    if (enumeration + max_elements >= fact)
        max_elements = fact - enumeration; // check if > or >=

    printf("%llu|%llu|%llu|%llu\n", max_elements, index, enumeration, fact);
    u8 indeces[16];
    u8 result[16];
    permutation_by_enumeration(enumeration, result, indeces, size, fact);

    u8 neuer_stapel_tmp[16];
    Bestes momentan_bestes;
    bestes_gefundene[index].bestes.laenge = 0;

    for (u64 i = 0; i < fact / size; i++)
    {
        print("%llu: %u (%u)\n", i, prior[i].laenge, prior[i].letzter_flip);
    }

    for (u64 i = 0; i < max_elements; i++)
    {
        if (i > 0)
        {
            // permutation_by_enumeration(++enumeration, result, indeces, size, fact);
            permutate(result, size, enumeration++);
        }

        momentan_bestes.letzter_flip = (u8)-1;
        bool sortiert = true;
        u8 letztes = (u8)-1;
        for (u8 j = 0; sortiert && j < size; j++)
        {
            if (result[j] > letztes)
                sortiert = false;
            else
                letztes = result[j];
        }

        if (sortiert)
            momentan_bestes.laenge = 0;
        else
        {
            momentan_bestes.laenge = (u8)-1;
            for (u8 flip = 0; !sortiert && flip < size; flip++)
            {
                wenden_und_essen(result, neuer_stapel_tmp, size, flip);

                sortiert = true;
                letztes = (u8)-1;
                for (u8 j = 0; sortiert && j < size - 1; j++)
                {
                    if (neuer_stapel_tmp[j] > letztes)
                        sortiert = false;
                    else
                        letztes = neuer_stapel_tmp[j];
                }
                u8 potenziel_beste_laenge = 1;
                if (!sortiert)
                {
                    u64 neue_enumeration = enumerate_permutation(neuer_stapel_tmp, indeces, size - 1);
                    u8 vorherige_laenge = prior[neue_enumeration].laenge;
                    potenziel_beste_laenge += vorherige_laenge;
                    print("%llu|%u: %u vs %u (%llu:%u)\n", enumeration, flip, potenziel_beste_laenge, momentan_bestes.laenge, neue_enumeration, vorherige_laenge);
                }
                if (potenziel_beste_laenge < momentan_bestes.laenge)
                {
                    momentan_bestes.laenge = potenziel_beste_laenge;
                    momentan_bestes.letzter_flip = flip;
                }
            }
        }
        if (momentan_bestes.laenge != (u8)-1)
        {
            current[enumeration] = momentan_bestes;
            if (momentan_bestes.laenge > bestes_gefundene[index].bestes.laenge)
            {
                bestes_gefundene[index].bestes = momentan_bestes;
                bestes_gefundene[index].enumeration = enumeration;
            }
        }
    }
}