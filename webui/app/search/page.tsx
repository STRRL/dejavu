"use client";

import { Card, CardMedia } from "@mui/material";
import { useSearchParams } from 'next/navigation'
import { useQuery } from '@tanstack/react-query'
import Link from "next/link";

export default function Search() {

  const searchParams = useSearchParams()
  const text = searchParams.get('text')

  const { data } = useQuery({
    queryKey: ['search', text],
    queryFn: async () => {
      const response = await fetch(`/api/search?text=${text}`)
      return await response.json() as {
        image_id: string
        texts: {
          id: number,
          image_id: string,
          text: string,
          left: number,
          top: number,
          width: number,
          height: number,
        }[]
      }[]
    }
  })

  return (
    <main>
      <div className='container m-auto'>
        <div className="p-8">
          <div className="pb-8">
            <p className="text-2xl">{`Result for "${text}":`}</p>
          </div>
          <div className="grid grid-cols-4 gap-4">
            {data?.slice(0, 20).map((item, i) => {
              const text_ids = item.texts.map(it => it.id).join(',');
              return (<div key={i}>
                <Link href={`/detail?image_id=${item.image_id}&text_ids=${text_ids}`}>
                  <Card>
                    <CardMedia
                      className="w-[24rem] h-[13.5rem]"
                      image={`/api/image?image_id=${item.image_id}&text_ids=${text_ids}`}>
                    </CardMedia>
                  </Card>
                </Link>

              </div>)
            }
            )}
          </div>
        </div>
      </div>
    </main>
  );
}
